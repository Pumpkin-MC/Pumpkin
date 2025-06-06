use pumpkin_data::BlockDirection;
use pumpkin_util::{
    math::position::BlockPos,
    random::{RandomGenerator, RandomImpl},
};
use serde::Deserialize;

use crate::ProtoChunk;

use super::CoralFeature;

#[derive(Deserialize)]
pub struct CoralTreeFeature;

impl CoralTreeFeature {
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
        let mut pos = pos;
        let i = random.next_bounded_i32(3) + 1;
        for _ in 0..i {
            if !CoralFeature::generate_coral_piece(chunk, random, &block, pos) {
                return true;
            }
            pos = pos.up();
        }
        let i = random.next_bounded_i32(3) + 2;

        // TODO: Shuffle
        let directions: Vec<_> = BlockDirection::horizontal()
            .into_iter()
            .take(i as usize)
            .collect();
        for dir in directions {
            pos = pos.offset(dir.to_offset());
            let times = random.next_bounded_i32(5) + 2;
            let mut m = 0;
            for n in 0..times {
                if !CoralFeature::generate_coral_piece(chunk, random, &block, pos) {
                    break;
                }
                pos = pos.up();
                m += 1;
                if n != 0 && (m < 2 || random.next_f32() >= 0.25) {
                    continue;
                }
                pos = pos.offset(dir.to_offset());
                m = 0;
            }
        }
        true
    }
}
