use crate::{generation::proto_chunk::GenerationCache, world::WorldPortalExt};
use pumpkin_data::{BlockDirection, tag};
use pumpkin_util::{
    math::position::BlockPos,
    random::{RandomGenerator, RandomImpl},
};

use super::CoralFeature;

pub struct CoralTreeFeature;

impl CoralTreeFeature {
    pub fn generate<T: GenerationCache>(
        chunk: &mut T,
        block_registry: &dyn WorldPortalExt,
        _min_y: i8,
        _height: u16,
        _feature: pumpkin_data::placed_feature::PlacedFeature,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        let block = CoralFeature::get_random_tag_entry(tag::Block::MINECRAFT_CORAL_BLOCKS, random);
        let mut cur_pos = pos;
        let trunk_height = random.next_bounded_i32(3) + 1;
        for _ in 0..trunk_height {
            if !CoralFeature::generate_coral_piece(chunk, block_registry, random, block, cur_pos) {
                return true;
            }
            cur_pos = cur_pos.up();
        }

        let trunk_top_pos = cur_pos;
        let n_branches = random.next_bounded_i32(3) + 2;

        let mut directions: Vec<BlockDirection> = BlockDirection::horizontal_worldgen()
            .iter()
            .copied()
            .map(BlockDirection::from_cardinal_direction)
            .collect();
        for i in (1..directions.len()).rev() {
            let j = random.next_bounded_i32(i as i32 + 1) as usize;
            directions.swap(i, j);
        }

        for &branch_direction in directions.iter().take(n_branches as usize) {
            let mut branch_pos = trunk_top_pos.offset(branch_direction.to_offset());
            let branch_height = random.next_bounded_i32(5) + 2;
            let mut segment_length = 0;

            for j in 0..branch_height {
                if !CoralFeature::generate_coral_piece(
                    chunk,
                    block_registry,
                    random,
                    block,
                    branch_pos,
                ) {
                    break;
                }
                segment_length += 1;
                branch_pos = branch_pos.up();
                if j == 0 || (segment_length >= 2 && random.next_f32() < 0.25) {
                    branch_pos = branch_pos.offset(branch_direction.to_offset());
                    segment_length = 0;
                }
            }
        }
        true
    }
}
