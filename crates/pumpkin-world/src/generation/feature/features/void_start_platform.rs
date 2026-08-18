use pumpkin_data::Block;
use pumpkin_util::{math::position::BlockPos, random::RandomGenerator};

use crate::generation::proto_chunk::GenerationCache;

pub struct VoidStartPlatformFeature;

impl VoidStartPlatformFeature {
    #[allow(clippy::unused_self)]
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        _min_y: i8,
        _height: u16,
        _feature: pumpkin_data::placed_feature::PlacedFeature,
        _random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        for dx in -1..=1 {
            for dz in -1..=1 {
                let target = BlockPos::new(pos.0.x + dx, pos.0.y, pos.0.z + dz);
                chunk.set_block_state(&target.0, Block::OBSIDIAN.default_state);
            }
        }
        true
    }
}
