use crate::{generation::proto_chunk::GenerationCache, world::WorldPortalExt};
use pumpkin_data::tag;
use pumpkin_util::{
    math::position::BlockPos,
    random::{RandomGenerator, RandomImpl},
};

use super::CoralFeature;

pub struct CoralMushroomFeature;

impl CoralMushroomFeature {
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

        let height = random.next_bounded_i32(3) + 3;
        let width = random.next_bounded_i32(3) + 3;
        let length = random.next_bounded_i32(3) + 3;
        let sink_value = random.next_bounded_i32(3) + 1;

        for x in 0..=width {
            for y in 0..=height {
                for z in 0..=length {
                    let cur_pos = BlockPos::new(pos.0.x + x, pos.0.y + y - sink_value, pos.0.z + z);

                    let condition_a = (x != 0 && x != width) || (y != 0 && y != height);
                    let condition_b = (z != 0 && z != length) || (y != 0 && y != height);
                    let condition_c = (x != 0 && x != width) || (z != 0 && z != length);
                    let condition_d =
                        x == 0 || x == width || y == 0 || y == height || z == 0 || z == length;

                    if condition_a
                        && condition_b
                        && condition_c
                        && condition_d
                        && random.next_f32() >= 0.1
                    {
                        CoralFeature::generate_coral_piece(
                            chunk,
                            block_registry,
                            random,
                            block,
                            cur_pos,
                        );
                    }
                }
            }
        }
        true
    }
}
