use crate::generation::proto_chunk::GenerationCache;
use pumpkin_data::{Block, BlockState};
use pumpkin_util::{
    math::position::BlockPos,
    random::{RandomGenerator, RandomImpl},
};

pub struct KelpFeature {
    // No extra parameters
}

impl KelpFeature {
    #[allow(clippy::too_many_arguments)]
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        _min_y: i8,
        _height: u16,
        _feature: &str,
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
                    // Can't place further but we have already placed at least one segment, try to put head below
                    let below = BlockPos::new(kelp_pos.0.x, kelp_pos.0.y - 1, kelp_pos.0.z);
                    // Check head block can survive and that below-below is not a kelp head
                    if GenerationCache::get_block_state(chunk, &below.0).to_block_id()
                        == Block::WATER
                        && GenerationCache::get_block_state(
                            chunk,
                            &BlockPos::new(below.0.x, below.0.y - 1, below.0.z).0,
                        )
                        .to_block_id()
                            != Block::KELP
                    {
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
