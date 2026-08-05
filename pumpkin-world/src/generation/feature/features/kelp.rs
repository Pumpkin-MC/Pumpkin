use crate::generation::proto_chunk::GenerationCache;
use pumpkin_data::{Block, BlockId, BlockState};
use pumpkin_util::{
    math::position::BlockPos,
    random::{RandomGenerator, RandomImpl},
};

/// Whether the last-placed segment of a growth column that got cut short (surface reached,
/// obstruction, etc.) can be capped with a kelp head. `below_id` is the block one below the
/// current failing position - either a `KELP_PLANT` body segment placed on the previous
/// iteration, or still-virgin water if nothing was ever placed (a one-block-deep water
/// column). `below_below_id` guards against stacking two heads on top of each other.
fn can_cap_with_head(below_id: BlockId, below_below_id: BlockId) -> bool {
    (below_id == BlockId::WATER || below_id == BlockId::KELP_PLANT)
        && below_below_id != BlockId::KELP
}

pub struct KelpFeature;

impl KelpFeature {
    #[allow(clippy::too_many_arguments)]
    pub fn generate<T: GenerationCache>(
        chunk: &mut T,
        _min_y: i8,
        _height: u16,
        _feature: pumpkin_data::placed_feature::PlacedFeature,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        let mut placed = 0;

        // Start on the ocean floor
        let y = chunk.ocean_floor_height_exclusive(pos.0.x, pos.0.z);
        let mut kelp_pos = BlockPos::new(pos.0.x, y, pos.0.z);

        // Must be in water
        if GenerationCache::get_block_state(chunk, &kelp_pos.0).to_block_id() == Block::WATER {
            let height_rand = 1 + random.next_bounded_i32(10);

            // Iterate from base up to height_rand
            for h in 0..=height_rand {
                // Check there is water at this position and one above
                if GenerationCache::get_block_state(chunk, &kelp_pos.0).to_block_id()
                    == Block::WATER
                    && GenerationCache::get_block_state(
                        chunk,
                        &BlockPos::new(kelp_pos.0.x, kelp_pos.0.y + 1, kelp_pos.0.z).0,
                    )
                    .to_block_id()
                        == Block::WATER
                {
                    // If this is the last iteration place the kelp head with age
                    if h == height_rand {
                        let age = random.next_bounded_i32(4) + 20;
                        // Clamp in case it goes past available states
                        let age = age.min((Block::KELP.states.len() - 1) as i32) as usize;
                        let state_id = Block::KELP.states[age].id;
                        let state = BlockState::from_id(state_id);
                        chunk.set_block_state(&kelp_pos.0, state);
                        placed += 1;
                    } else {
                        // Place kelp plant body
                        let state_id = Block::KELP_PLANT.default_state.id;
                        let state = BlockState::from_id(state_id);
                        chunk.set_block_state(&kelp_pos.0, state);
                    }
                } else if h > 0 {
                    // Can't place further but we have already placed at least one segment, try to put
                    // head below. `below` is the position the previous iteration wrote to: a
                    // KELP_PLANT body segment in the common case, or still-virgin WATER if the water
                    // column was only one block deep and nothing was ever placed.
                    let below = BlockPos::new(kelp_pos.0.x, kelp_pos.0.y - 1, kelp_pos.0.z);
                    let below_id = GenerationCache::get_block_state(chunk, &below.0).to_block_id();
                    let below_below_id = GenerationCache::get_block_state(
                        chunk,
                        &BlockPos::new(below.0.x, below.0.y - 1, below.0.z).0,
                    )
                    .to_block_id();
                    if can_cap_with_head(below_id, below_below_id) {
                        let age = random.next_bounded_i32(4) + 20;
                        let age = age.min((Block::KELP.states.len() - 1) as i32) as usize;
                        let state_id = Block::KELP.states[age].id;
                        let state = BlockState::from_id(state_id);
                        chunk.set_block_state(&below.0, state);
                        placed += 1;
                    }
                    break;
                }
                kelp_pos = BlockPos::new(kelp_pos.0.x, kelp_pos.0.y + 1, kelp_pos.0.z);
            }
        }

        placed > 0
    }
}

#[cfg(test)]
mod tests {
    use super::can_cap_with_head;
    use pumpkin_data::BlockId;

    #[test]
    fn caps_the_last_body_segment_placed_on_the_previous_iteration() {
        // The common case this fix targets: growth was cut short after at least one
        // KELP_PLANT body segment was already written below the failing position.
        assert!(can_cap_with_head(BlockId::KELP_PLANT, BlockId::STONE));
    }

    #[test]
    fn caps_a_still_virgin_water_column_one_block_deep() {
        assert!(can_cap_with_head(BlockId::WATER, BlockId::STONE));
    }

    #[test]
    fn refuses_to_stack_a_head_directly_on_another_head() {
        assert!(!can_cap_with_head(BlockId::KELP_PLANT, BlockId::KELP));
        assert!(!can_cap_with_head(BlockId::WATER, BlockId::KELP));
    }

    #[test]
    fn refuses_when_below_is_neither_water_nor_a_body_segment() {
        assert!(!can_cap_with_head(BlockId::STONE, BlockId::STONE));
        assert!(!can_cap_with_head(BlockId::AIR, BlockId::STONE));
    }
}
