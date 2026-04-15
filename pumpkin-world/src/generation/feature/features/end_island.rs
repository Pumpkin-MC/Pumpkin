use pumpkin_data::Block;
use pumpkin_util::{
    math::{position::BlockPos, vector3::Vector3},
    random::{RandomGenerator, RandomImpl},
};

use crate::generation::proto_chunk::GenerationCache;

pub struct EndIslandFeature {}

impl EndIslandFeature {
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        _min_y: i8,
        _height: u16,
        _feature: &str,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        let mut size = random.next_bounded_i32(3) as f32 + 4.0;

        let mut dy = 0i32;
        while size > 0.5 {
            let start = size.copysign(-1.0).floor() as i32;
            let end = size.ceil() as i32;
            let radius_sq = (size + 1.0) * (size + 1.0);

            for dx in start..=end {
                for dz in start..=end {
                    if (dx * dx + dz * dz) as f32 <= radius_sq {
                        let target = Vector3::new(pos.0.x + dx, pos.0.y + dy, pos.0.z + dz);
                        chunk.set_block_state(&target, Block::END_STONE.default_state);
                    }
                }
            }

            size -= random.next_bounded_i32(2) as f32 + 0.5;
            dy -= 1;
        }

        true
    }
}
