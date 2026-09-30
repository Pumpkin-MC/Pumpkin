use pumpkin_data::Block;
use pumpkin_util::{
    math::{position::BlockPos, vector3::Vector3},
    random::RandomGenerator,
};

use crate::generation::proto_chunk::GenerationCache;

pub struct VoidStartPlatformFeature;

impl VoidStartPlatformFeature {
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        _min_y: i8,
        _height: u16,
        _feature: pumpkin_data::placed_feature::PlacedFeature,
        _random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        let chunk_x = pos.0.x >> 4;
        let chunk_z = pos.0.z >> 4;

        if chunk_x.abs().max(chunk_z.abs()) > 1 {
            return true;
        }

        let platform_y = pos.0.y + 3;
        let min_x = chunk_x * 16;
        let max_x = min_x + 15;
        let min_z = chunk_z * 16;
        let max_z = min_z + 15;

        for z in min_z..=max_z {
            for x in min_x..=max_x {
                if (8 - x).abs().max((8 - z).abs()) <= 16 {
                    let target = Vector3::new(x, platform_y, z);
                    let state = if x == 8 && z == 8 {
                        Block::COBBLESTONE.default_state
                    } else {
                        Block::STONE.default_state
                    };
                    chunk.set_block_state(&target, state);
                }
            }
        }

        true
    }
}
