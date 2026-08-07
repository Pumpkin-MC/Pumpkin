use pumpkin_data::Block;
use pumpkin_util::{math::position::BlockPos, random::RandomGenerator};

use crate::generation::proto_chunk::GenerationCache;

pub struct BonusChestFeature;

impl BonusChestFeature {
    pub fn generate<T: GenerationCache>(
        chunk: &mut T,
        _min_y: i8,
        _height: u16,
        _feature: pumpkin_data::placed_feature::PlacedFeature,
        _random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        let mut block_pos = pos;
        while chunk.is_air(&block_pos.0) && block_pos.0.y > chunk.bottom_y() as i32 + 2 {
            block_pos = block_pos.down();
        }

        let chest_pos = block_pos.up();
        if !chunk.is_air(&chest_pos.0) {
            return false;
        }

        chunk.set_block_state(&chest_pos.0, Block::CHEST.default_state);

        let torch_offsets = [(1, 0, 0), (-1, 0, 0), (0, 0, 1), (0, 0, -1)];
        for &(dx, dy, dz) in &torch_offsets {
            let t_pos = BlockPos::new(chest_pos.0.x + dx, chest_pos.0.y + dy, chest_pos.0.z + dz);
            if chunk.is_air(&t_pos.0) {
                chunk.set_block_state(&t_pos.0, Block::TORCH.default_state);
            }
        }
        true
    }
}
