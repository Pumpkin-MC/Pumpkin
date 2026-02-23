use pumpkin_data::Block;
use pumpkin_util::{math::position::BlockPos, random::RandomGenerator};

use crate::generation::proto_chunk::GenerationCache;
use crate::world::BlockRegistryExt;

pub struct EndPlatformFeature;

impl EndPlatformFeature {
    #[expect(clippy::too_many_arguments)]
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        _block_registry: &dyn BlockRegistryExt,
        _min_y: i8,
        _height: u16,
        _feature: &str, // This placed feature
        _random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        for _ in -2..2 {
            for _ in -2..2 {
                for t in -1..3 {
                    let state = if t == -1 {
                        Block::OBSIDIAN.default_state
                    } else {
                        Block::AIR.default_state
                    };
                    if GenerationCache::get_block_state(chunk, &pos.0).0 == state.id {
                        continue;
                    }
                    chunk.set_block_state(&pos.0, state);
                }
            }
        }
        true
    }
}
