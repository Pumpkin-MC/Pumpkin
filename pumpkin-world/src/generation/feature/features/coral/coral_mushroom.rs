use pumpkin_util::{
    math::{position::BlockPos, vector3::Vector3},
    random::{RandomGenerator, RandomImpl},
};
use serde::Deserialize;

use crate::ProtoChunk;

use super::CoralFeature;

#[derive(Deserialize)]
pub struct CoralMushroomFeature;

impl CoralMushroomFeature {
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

        let i = random.next_bounded_i32(3) + 3;
        let j = random.next_bounded_i32(3) + 3;
        let k = random.next_bounded_i32(3) + 3;
        let l = random.next_bounded_i32(3) + 1;

        for m in 0..=j {
            for n in 0..=i {
                for o in 0..=k {
                    let mut pos = pos;
                    pos = pos.offset(Vector3::new(pos.0.x + m, pos.0.y + n, pos.0.z + o));
                    pos = pos.down_height(l);

                    let condition_a = (m != 0 && m != j) || (n != 0 && n != i);
                    let condition_b = (o != 0 && o != k) || (n != 0 && n != i);
                    let condition_c = (m != 0 && m != j) || (o != 0 && o != k);
                    let condition_d = m == 0 || m == j || n == 0 || n == i || o == 0 || o == k;
                    let random_check = random.next_f32() < 0.1f32;

                    if !((condition_a && condition_b && condition_c && condition_d)
                        && !random_check
                        && CoralFeature::generate_coral_piece(chunk, random, &block, pos))
                    {
                        continue;
                    }
                }
            }
        }
        true
    }
}
