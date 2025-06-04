use pumpkin_data::BlockDirection;
use pumpkin_util::{
    math::position::BlockPos,
    random::{RandomGenerator, RandomImpl},
};
use serde::Deserialize;

use crate::ProtoChunk;

use super::CoralFeature;

#[derive(Deserialize)]
pub struct CoralClawFeature;

impl CoralClawFeature {
    pub fn generate(
        &self,
        chunk: &mut ProtoChunk,
        _min_y: i8,
        _height: u16,
        _feature: &str, // This placed feature
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        // First lets get a random coral
        let block = CoralFeature::get_random_tag_entry("minecraft:coral_blocks", random);
        if !CoralFeature::generate_coral_piece(chunk, random, &block, pos) {
            return false;
        }
        let i = random.next_bounded_i32(2) + 2;
        let direction = BlockDirection::horizontal()
            [random.next_bounded_i32(BlockDirection::horizontal().len() as i32 - 1) as usize];
        // TODO: Shuffle
        let directions: Vec<_> = BlockDirection::horizontal()
            .into_iter()
            .take(i as usize)
            .collect();
        'block0: for direction2 in directions {
            let mut pos = pos;
            let j = random.next_bounded_i32(2) + 1;
            pos = pos.offset(direction2.to_offset());

            let direction3;
            let k;

            if direction2 == direction {
                direction3 = direction;
                k = random.next_bounded_i32(3) + 2;
            } else {
                pos = pos.up();
                let _directions = [direction2, BlockDirection::Up];
                direction3 = direction2; // TODO: make this random
                k = random.next_bounded_i32(3) + 5;
            }

            for _ in 0..j {
                if !CoralFeature::generate_coral_piece(chunk, random, &block, pos) {
                    break;
                }
                pos = pos.offset(direction3.to_offset());
            }

            pos = pos.offset(direction3.to_offset());
            pos = pos.up();

            for _l in 0..k {
                pos = pos.offset(direction.opposite().to_offset());
                if !CoralFeature::generate_coral_piece(chunk, random, &block, pos) {
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
