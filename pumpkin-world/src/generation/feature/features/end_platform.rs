use pumpkin_data::Block;
use pumpkin_util::{
    math::{position::BlockPos, vector3::Vector3},
    random::RandomGenerator,
};

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
        for dx in -2i32..=2 {
            for dz in -2i32..=2 {
                for dy in -1i32..3 {
                    let state = if dy == -1 {
                        Block::OBSIDIAN.default_state
                    } else {
                        Block::AIR.default_state
                    };
                    let target = Vector3::new(pos.0.x + dx, pos.0.y + dy, pos.0.z + dz);
                    chunk.set_block_state(&target, state);
                }
            }
        }
        true
    }
}
