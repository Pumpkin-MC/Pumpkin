use crate::{generation::proto_chunk::GenerationCache, world::WorldPortalExt};
use pumpkin_data::{BlockDirection, tag};
use pumpkin_util::{
    math::position::BlockPos,
    random::{RandomGenerator, RandomImpl},
};

use super::CoralFeature;

pub struct CoralClawFeature;

impl CoralClawFeature {
    #[allow(clippy::too_many_arguments)]
    pub fn generate<T: GenerationCache>(
        chunk: &mut T,
        block_registry: &dyn WorldPortalExt,
        _min_y: i8,
        _height: u16,
        _feature: pumpkin_data::placed_feature::PlacedFeature, // This placed feature
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        // First lets get a random coral
        let block = CoralFeature::get_random_tag_entry(tag::Block::MINECRAFT_CORAL_BLOCKS, random);
        if !CoralFeature::generate_coral_piece(chunk, block_registry, random, block, pos) {
            return false;
        }
        let i = random.next_bounded_i32(2) + 2;
        let direction = BlockDirection::horizontal()
            [random.next_bounded_i32(BlockDirection::horizontal().len() as i32 - 1) as usize];
        // TODO: Shuffle
        let directions = BlockDirection::horizontal().into_iter().take(i as usize);
        'block0: for direction2 in directions {
            let mut pos = pos;
            let j = random.next_bounded_i32(2) + 1;
            pos = pos.offset(direction2.to_offset());

            let branch_direction;

            let k = if direction2 == direction {
                branch_direction = direction;
                random.next_bounded_i32(3) + 2
            } else {
                pos = pos.up();
                //let _directions = [direction2, BlockDirection::Up];
                branch_direction = direction2; // TODO: make this random
                random.next_bounded_i32(3) + 5
            };

            for _ in 0..j {
                if !CoralFeature::generate_coral_piece(chunk, block_registry, random, block, pos) {
                    break;
                }
                pos = pos.offset(branch_direction.to_offset());
            }

            pos = pos.offset(branch_direction.to_offset());
            pos = pos.up();

            for _l in 0..k {
                pos = pos.offset(direction.opposite().to_offset());
                if !CoralFeature::generate_coral_piece(chunk, block_registry, random, block, pos) {
                    continue 'block0;
                }
                if random.next_f32() < 0.25 {
                    pos = pos.up();
                }
            }
        }
        true
    }
}
