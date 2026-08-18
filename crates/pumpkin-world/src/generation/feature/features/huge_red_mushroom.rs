use pumpkin_data::Block;
use pumpkin_util::{math::position::BlockPos, random::RandomGenerator, random::RandomImpl};

use crate::generation::proto_chunk::GenerationCache;

pub struct HugeRedMushroomFeature;

impl HugeRedMushroomFeature {
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
        let height = random.next_bounded_i32(3) + 5;

        for i in 0..height {
            let stem_pos = BlockPos::new(pos.0.x, pos.0.y + i, pos.0.z);
            chunk.set_block_state(&stem_pos.0, Block::MUSHROOM_STEM.default_state);
        }

        let cap_y = pos.0.y + height;
        for dy in 0..=2 {
            let radius = if dy == 0 { 2 } else { 1 };
            for dx in -radius..=radius {
                for dz in -radius..=radius {
                    let cap_pos = BlockPos::new(pos.0.x + dx, cap_y + dy, pos.0.z + dz);
                    chunk.set_block_state(&cap_pos.0, Block::RED_MUSHROOM_BLOCK.default_state);
                }
            }
        }
        true
    }
}
