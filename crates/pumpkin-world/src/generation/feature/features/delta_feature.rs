use pumpkin_data::{BlockDirection, BlockId, BlockState};
use pumpkin_util::{
    math::{int_provider::IntProvider, position::BlockPos},
    random::{RandomGenerator, RandomImpl},
};

use crate::generation::proto_chunk::GenerationCache;

pub struct DeltaFeatureFeature {
    pub contents: &'static BlockState,
    pub rim: &'static BlockState,
    pub size: IntProvider,
    pub rim_size: IntProvider,
}

impl DeltaFeatureFeature {
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        _min_y: i8,
        _height: u16,
        _feature: pumpkin_data::placed_feature::PlacedFeature,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        let mut any_placed = false;
        let spawn_rim = random.next_f64() < 0.9;
        let rim_x = if spawn_rim {
            self.rim_size.get(random)
        } else {
            0
        };
        let rim_z = if spawn_rim {
            self.rim_size.get(random)
        } else {
            0
        };
        let has_rim = spawn_rim && rim_x != 0 && rim_z != 0;
        let radius_x = self.size.get(random);
        let radius_z = self.size.get(random);
        let radius_limit = radius_x.max(radius_z);

        for pos_iter in BlockPos::iterate_outwards(pos, radius_x, 0, radius_z) {
            if pos_iter.manhattan_distance(pos) > radius_limit {
                break;
            }

            if Self::is_clear(chunk, pos_iter, self.contents) {
                if has_rim {
                    any_placed = true;
                    chunk.set_block_state(&pos_iter.0, self.rim);
                }

                let pos_offset = pos_iter.add(rim_x, 0, rim_z);
                if Self::is_clear(chunk, pos_offset, self.contents) {
                    any_placed = true;
                    chunk.set_block_state(&pos_offset.0, self.contents);
                }
            }
        }

        any_placed
    }

    fn is_clear<T: GenerationCache>(
        chunk: &T,
        pos: BlockPos,
        contents: &'static BlockState,
    ) -> bool {
        let state = GenerationCache::get_block_state(chunk, &pos.0);
        if state.to_block_id() == contents.id.to_block_id() {
            return false;
        }

        if is_cannot_replace(state.to_block_id()) {
            return false;
        }

        for dir in BlockDirection::all() {
            let neighbor_state =
                GenerationCache::get_block_state(chunk, &pos.offset(dir.to_offset()).0);
            let is_air = neighbor_state.to_state().is_air();
            if (is_air && dir != BlockDirection::Up) || (!is_air && dir == BlockDirection::Up) {
                return false;
            }
        }

        true
    }
}

const fn is_cannot_replace(id: BlockId) -> bool {
    matches!(
        id,
        BlockId::BEDROCK
            | BlockId::NETHER_BRICKS
            | BlockId::NETHER_BRICK_FENCE
            | BlockId::NETHER_BRICK_STAIRS
            | BlockId::NETHER_WART
            | BlockId::CHEST
            | BlockId::SPAWNER
    )
}
