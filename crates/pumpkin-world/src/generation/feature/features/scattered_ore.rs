use pumpkin_util::{
    math::position::BlockPos,
    random::{RandomGenerator, RandomImpl},
};

use crate::generation::proto_chunk::GenerationCache;
use crate::world::WorldPortalExt;

use super::ore::{OreFeature, OreTarget};

pub struct ScatteredOreFeature {
    pub size: i32,
    pub discard_chance_on_air_exposure: f32,
    pub targets: Vec<OreTarget>,
}

impl ScatteredOreFeature {
    #[expect(clippy::too_many_arguments)]
    pub fn generate<T: GenerationCache>(
        &self,
        chunk: &mut T,
        _block_registry: &dyn WorldPortalExt,
        _min_y: i8,
        _height: u16,
        _feature: pumpkin_data::placed_feature::PlacedFeature,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        let count = random.next_bounded_i32(self.size + 1);
        for i in 0..count {
            let max_dist = i.min(7);
            let offset_x = Self::get_random_offset(random, max_dist);
            let offset_y = Self::get_random_offset(random, max_dist);
            let offset_z = Self::get_random_offset(random, max_dist);

            let target_pos = pos.add(offset_x, offset_y, offset_z);

            if chunk.out_of_height(target_pos.0.y as i16) {
                continue;
            }

            let block_state = GenerationCache::get_block_state(chunk, &target_pos.0);

            for target in &self.targets {
                if OreFeature::should_place(
                    self.discard_chance_on_air_exposure,
                    chunk,
                    block_state,
                    random,
                    target,
                    &target_pos,
                ) {
                    chunk.set_block_state(&target_pos.0, target.state);
                    break;
                }
            }
        }

        true
    }

    fn get_random_offset(random: &mut RandomGenerator, max_dist: i32) -> i32 {
        let f1 = random.next_f32();
        let f2 = random.next_f32();
        ((f1 - f2) * max_dist as f32).round() as i32
    }
}
