use pumpkin_data::Block;
use pumpkin_util::{math::position::BlockPos, random::RandomGenerator};

use crate::generation::proto_chunk::GenerationCache;

pub struct EndGatewayFeature;

impl EndGatewayFeature {
    pub fn generate<T: GenerationCache>(
        chunk: &mut T,
        _min_y: i8,
        _height: u16,
        _feature: pumpkin_data::placed_feature::PlacedFeature,
        _random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        for dx in -1..=1 {
            for dy in -1..=1 {
                for dz in -1..=1 {
                    let target = BlockPos::new(pos.0.x + dx, pos.0.y + dy, pos.0.z + dz);
                    let is_center = dx == 0 && dy == 0 && dz == 0;
                    let is_corner = dx.abs() + dy.abs() + dz.abs() == 3;

                    if is_center {
                        chunk.set_block_state(&target.0, Block::END_GATEWAY.default_state);
                    } else if !is_corner {
                        chunk.set_block_state(&target.0, Block::BEDROCK.default_state);
                    }
                }
            }
        }
        true
    }
}
