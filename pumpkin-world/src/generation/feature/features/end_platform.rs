use pumpkin_data::Block;
use pumpkin_util::{math::position::BlockPos, random::RandomGenerator};
use serde::Deserialize;

use crate::{ProtoChunk, world::BlockRegistryExt};

#[derive(Deserialize)]
pub struct EndPlatformFeature;

impl EndPlatformFeature {
    #[expect(clippy::too_many_arguments)]
    pub fn generate(
        &self,
        chunk: &mut ProtoChunk<'_>,
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
                    if chunk.get_block_state(&pos.0).0 == state.id {
                        continue;
                    }
                    chunk.set_block_state(&pos.0, &state);
                }
            }
        }
        true
    }
}
