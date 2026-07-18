use pumpkin_data::Block;
use pumpkin_util::{math::position::BlockPos, random::RandomGenerator, random::RandomImpl};

use crate::generation::proto_chunk::GenerationCache;

pub struct HugeFungusFeature;

impl HugeFungusFeature {
    #[allow(clippy::unused_self)]
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        _min_y: i8,
        _height: u16,
        _feature: pumpkin_data::placed_feature::PlacedFeature,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        let height = random.next_bounded_i32(4) + 6;
        let is_warped = random.next_bool();

        let stem_state = if is_warped {
            Block::WARPED_STEM.default_state
        } else {
            Block::CRIMSON_STEM.default_state
        };
        let wart_state = if is_warped {
            Block::WARPED_WART_BLOCK.default_state
        } else {
            Block::NETHER_WART_BLOCK.default_state
        };

        for i in 0..height {
            let stem_pos = BlockPos::new(pos.0.x, pos.0.y + i, pos.0.z);
            chunk.set_block_state(&stem_pos.0, stem_state);
        }

        let cap_y = pos.0.y + height - 2;
        for dy in 0..=3 {
            let radius = if dy == 0 || dy == 3 { 1 } else { 2 };
            for dx in -radius..=radius {
                for dz in -radius..=radius {
                    let cap_pos = BlockPos::new(pos.0.x + dx, cap_y + dy, pos.0.z + dz);
                    let block_state = if (dx == 0 || dz == 0) && dy == 1 && random.next_f32() < 0.3
                    {
                        Block::SHROOMLIGHT.default_state
                    } else {
                        wart_state
                    };
                    chunk.set_block_state(&cap_pos.0, block_state);
                }
            }
        }
        true
    }
}
