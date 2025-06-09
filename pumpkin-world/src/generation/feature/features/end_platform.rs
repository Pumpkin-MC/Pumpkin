use pumpkin_data::{Block, block_properties::get_state_by_state_id};
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
                    let block = if t == -1 {
                        Block::OBSIDIAN.default_state_id
                    } else {
                        Block::AIR.default_state_id
                    };
                    let state = get_state_by_state_id(block).unwrap();
                    if chunk.get_block_state(&pos.0).state_id == state.id {
                        continue;
                    }
                    chunk.set_block_state(&pos.0, &state);
                }
            }
        }
        true
    }
}
