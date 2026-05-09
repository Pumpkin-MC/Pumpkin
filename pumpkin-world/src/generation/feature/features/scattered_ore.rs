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
        _feature: &str,
        random: &mut RandomGenerator,
        pos: BlockPos,
    ) -> bool {
        let count = random.next_bounded_i32(self.size + 1);
        let mut placed = false;

        for _ in 0..count {
            let offset_x = self.get_random_offset(random);
            let offset_y = self.get_random_offset(random);
            let offset_z = self.get_random_offset(random);

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
                    placed = true;
                    break;
                }
            }
        }

        placed
    }

    fn get_random_offset(&self, random: &mut RandomGenerator) -> i32 {
        let f1 = random.next_f32();
        let f2 = random.next_f32();
        ((f1 - f2) * self.size as f32).round() as i32
    }
}
