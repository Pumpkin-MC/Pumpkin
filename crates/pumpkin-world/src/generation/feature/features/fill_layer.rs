use pumpkin_data::BlockState;
use pumpkin_util::{math::position::BlockPos, random::RandomGenerator};

use crate::generation::proto_chunk::GenerationCache;

pub struct FillLayerFeature {
    pub height: i32,
    pub state: &'static BlockState,
}

impl FillLayerFeature {
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        min_y: i8,
        _height: u16,
        _feature: pumpkin_data::placed_feature::PlacedFeature,
        _random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        let y = min_y as i32 + self.height;
        for dx in 0..16 {
            for dz in 0..16 {
                let target = BlockPos::new(pos.0.x + dx, y, pos.0.z + dz);
                if chunk.is_air(&target.0) {
                    chunk.set_block_state(&target.0, self.state);
                }
            }
        }
        true
    }
}
