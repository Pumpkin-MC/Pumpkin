use pumpkin_util::{math::position::BlockPos, random::RandomGenerator, random::RandomImpl};

use crate::generation::proto_chunk::GenerationCache;

pub struct LargeDripstoneFeature;

impl LargeDripstoneFeature {
    #[allow(clippy::unused_self)]
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        let height = random.next_bounded_i32(8) + 4;
        let mut generated = false;

        let mut current_pos = pos;
        while chunk.is_air(&current_pos.0) && current_pos.0.y > chunk.bottom_y() as i32 + 2 {
            current_pos = current_pos.down();
        }

        for i in 0..height {
            let target = BlockPos::new(current_pos.0.x, current_pos.0.y + i, current_pos.0.z);
            if super::gen_dripstone(chunk, target) {
                generated = true;
            }
        }
        generated
    }
}
