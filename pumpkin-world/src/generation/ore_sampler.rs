use pumpkin_data::{Block, BlockState};
use pumpkin_util::{
    math::clamped_map,
    random::{RandomDeriver, RandomDeriverImpl, RandomImpl},
};

use crate::generation::noise_router::chunk_noise_router::ChunkNoiseRouter;

use super::noise_router::{
    chunk_density_function::ChunkNoiseFunctionSampleOptions, density_function::NoisePos,
};

pub struct OreVeinSampler {
    random_deriver: RandomDeriver,
}

impl OreVeinSampler {
    pub fn new(random_deriver: RandomDeriver) -> Self {
        Self { random_deriver }
    }

    pub fn sample(
        &self,
        router: &mut ChunkNoiseRouter,
        pos: &impl NoisePos,
        sample_options: &ChunkNoiseFunctionSampleOptions,
    ) -> Option<BlockState> {
        let vein_toggle = router.vein_toggle(pos, sample_options);
        let vein_type: &VeinType = if vein_toggle > 0f64 {
            &vein_type::COPPER
        } else {
            &vein_type::IRON
        };

        let block_y = pos.y();
        let max_to_y = vein_type.max_y - block_y;
        let y_to_min = block_y - vein_type.min_y;
        if (max_to_y >= 0) && (y_to_min >= 0) {
            let closest_to_bound = max_to_y.min(y_to_min);
            let mapped_diff = clamped_map(closest_to_bound as f64, 0f64, 20f64, -0.2f64, 0f64);
            let abs_sample = vein_toggle.abs();
            if abs_sample + mapped_diff >= 0.4f32 as f64 {
                let mut random = self.random_deriver.split_pos(pos.x(), block_y, pos.z());

                let vein_ridged_sample = router.vein_ridged(pos, sample_options);
                if random.next_f32() <= 0.7f32 && vein_ridged_sample < 0f64 {
                    let clamped_sample = clamped_map(
                        abs_sample,
                        0.4f32 as f64,
                        0.6f32 as f64,
                        0.1f32 as f64,
                        0.3f32 as f64,
                    );

                    let vein_gap = router.vein_gap(pos, sample_options);
                    return if (random.next_f32() as f64) < clamped_sample
                        && vein_gap > (-0.3f32 as f64)
                    {
                        Some(if random.next_f32() < 0.02f32 {
                            vein_type.raw_ore.default_state.clone()
                        } else {
                            vein_type.ore.default_state.clone()
                        })
                    } else {
                        Some(vein_type.stone.default_state.clone())
                    };
                }
            }
        }
        None
    }
}

pub struct VeinType {
    ore: Block,
    raw_ore: Block,
    stone: Block,
    min_y: i32,
    max_y: i32,
}

// One of the victims of removing compile time blocks
pub mod vein_type {
    use pumpkin_data::Block;

    use super::*;
    pub const COPPER: VeinType = VeinType {
        ore: Block::COPPER_ORE,
        raw_ore: Block::RAW_COPPER_BLOCK,
        stone: Block::GRANITE,
        min_y: 0,
        max_y: 50,
    };
    pub const IRON: VeinType = VeinType {
        ore: Block::DEEPSLATE_IRON_ORE,
        raw_ore: Block::RAW_IRON_BLOCK,
        stone: Block::TUFF,
        min_y: -60,
        max_y: -8,
    };
    pub const MIN_Y: i32 = IRON.min_y;
    pub const MAX_Y: i32 = COPPER.max_y;
}
