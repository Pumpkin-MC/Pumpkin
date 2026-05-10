use pumpkin_data::{Block, BlockDirection};
use pumpkin_util::{
    math::position::BlockPos,
    random::{RandomGenerator, RandomImpl},
};

use crate::generation::proto_chunk::GenerationCache;

pub struct GlowstoneBlobFeature {}

impl GlowstoneBlobFeature {
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        _min_y: i8,
        _height: u16,
        _feature: &str,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        if !chunk.is_air(&pos.0) {
            return false;
        }

        let above_id = GenerationCache::get_block_state(chunk, &pos.up().0).to_block_id();
        if above_id != Block::NETHERRACK.id
            && above_id != Block::BASALT.id
            && above_id != Block::BLACKSTONE.id
        {
            return false;
        }

        chunk.set_block_state(&pos.0, Block::GLOWSTONE.default_state);

        for _ in 0..1500 {
            let place_pos = pos.add(
                random.next_bounded_i32(8) - random.next_bounded_i32(8),
                -random.next_bounded_i32(12),
                random.next_bounded_i32(8) - random.next_bounded_i32(8),
            );

            if chunk.is_air(&place_pos.0) {
                let mut neighbours = 0u8;

                for dir in BlockDirection::all() {
                    let neighbor = place_pos.0.add(&dir.to_offset());
                    if GenerationCache::get_block_state(chunk, &neighbor).to_block_id()
                        == Block::GLOWSTONE.id
                    {
                        neighbours += 1;
                    }
                    if neighbours > 1 {
                        break;
                    }
                }

                if neighbours == 1 {
                    chunk.set_block_state(&place_pos.0, Block::GLOWSTONE.default_state);
                }
            }
        }

        true
    }
}
