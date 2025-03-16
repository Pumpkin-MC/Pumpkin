use pumpkin_data::block::BlockState;
use pumpkin_macros::block_state;
use pumpkin_util::{
    math::vector3::Vector3,
    random::{RandomDeriver, RandomGenerator},
};

use crate::{
    block::ChunkBlockState,
    generation::{
        noise::perlin::DoublePerlinNoiseSampler,
        noise_router::proto_noise_router::DoublePerlinNoiseBuilder,
    },
};

pub struct SurfaceTerrainBuilder {
    terracotta_bands: Box<[ChunkBlockState]>,
    terracotta_bands_offset_noise: DoublePerlinNoiseSampler,
}

impl SurfaceTerrainBuilder {
    pub fn new(
        noise_builder: &mut DoublePerlinNoiseBuilder,
        random_deriver: &RandomDeriver,
    ) -> Self {
        Self {
            terracotta_bands: Self::create_terracotta_bands(
                random_deriver.split_string("minecraft:clay_bands"),
            ),
            terracotta_bands_offset_noise: noise_builder
                .get_noise_sampler_for_id("clay_bands_offset"),
        }
    }

    const ORANGE_TERRACOTTA: ChunkBlockState = block_state!("orange_terracotta");
    const YELLOW_TERRACOTTA: ChunkBlockState = block_state!("yellow_terracotta");
    const BROWN_TERRACOTTA: ChunkBlockState = block_state!("brown_terracotta");
    const RED_TERRACOTTA: ChunkBlockState = block_state!("red_terracotta");
    const WHITE_TERRACOTTA: ChunkBlockState = block_state!("white_terracotta");
    const LIGHT_GRAY_TERRACOTTA: ChunkBlockState = block_state!("light_gray_terracotta");

    fn create_terracotta_bands(mut random: RandomGenerator) -> Box<[ChunkBlockState]> {
        let mut block_states = [ChunkBlockState::AIR; 192];

        let mut i = 0;
        while i < block_states.len() {
            i += random.next_bounded_i32(5) as usize;
            if i >= block_states.len() {
                break;
            }
            block_states[i] = Self::ORANGE_TERRACOTTA;
        }

        Self::add_terracotta_bands(&mut random, &mut block_states, 1, Self::YELLOW_TERRACOTTA);
        Self::add_terracotta_bands(&mut random, &mut block_states, 2, Self::BROWN_TERRACOTTA);
        Self::add_terracotta_bands(&mut random, &mut block_states, 1, Self::RED_TERRACOTTA);

        let band_count = random.next_bounded_i32(15);
        let mut current_band = 0;
        let mut index = 0;

        while current_band < band_count && index < block_states.len() {
            block_states[index] = Self::WHITE_TERRACOTTA;

            if index > 0 && random.next_bool() {
                block_states[index - 1] = Self::LIGHT_GRAY_TERRACOTTA;
            }

            if index + 1 < block_states.len() && random.next_bool() {
                block_states[index + 1] = Self::LIGHT_GRAY_TERRACOTTA;
            }

            index += random.next_bounded_i32(19) as usize;
            current_band += 1;
        }

        Box::new(block_states)
    }

    fn add_terracotta_bands(
        random: &mut RandomGenerator,
        terracotta_bands: &mut [ChunkBlockState],
        min_band_size: i32,
        state: ChunkBlockState,
    ) {
        let band_count = random.next_inbetween_i32(6, 15);

        for _ in 0..band_count {
            let band_width = min_band_size + random.next_bounded_i32(3);
            let start_index = random.next_bounded_i32(terracotta_bands.len() as i32);

            for m in 0..band_width {
                if start_index + m < terracotta_bands.len() as i32 {
                    terracotta_bands[(start_index + m) as usize] = state;
                } else {
                    break; // Stop if we reach the end of the array
                }
            }
        }
    }

    pub fn get_terracotta_block(&self, pos: &Vector3<i32>) -> ChunkBlockState {
        let offset = (self
            .terracotta_bands_offset_noise
            .sample(pos.x as f64, 0.0, pos.z as f64)
            * 4.0)
            .round() as i32;
        let offset = pos.y + offset;
        self.terracotta_bands[((offset as usize + self.terracotta_bands.len())
            % self.terracotta_bands.len()) as usize]
    }
}
