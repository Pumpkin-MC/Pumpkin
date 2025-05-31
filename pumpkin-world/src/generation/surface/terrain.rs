use pumpkin_data::{BlockState, chunk::Biome};
use pumpkin_macros::default_block_state;
use pumpkin_util::{
    math::vector3::Vector3,
    random::{RandomDeriver, RandomDeriverImpl, RandomGenerator, RandomImpl},
};

use crate::{
    ProtoChunk,
    block::RawBlockState,
    generation::{
        chunk_noise::WATER_BLOCK, height_limit::HeightLimitView,
        noise::perlin::DoublePerlinNoiseSampler,
        noise_router::proto_noise_router::DoublePerlinNoiseBuilder,
    },
};

pub struct SurfaceTerrainBuilder {
    // Badlands stuff
    terracotta_bands: Box<[RawBlockState]>,
    terracotta_bands_offset_noise: DoublePerlinNoiseSampler,
    badlands_pillar_noise: DoublePerlinNoiseSampler,
    badlands_surface_noise: DoublePerlinNoiseSampler,
    badlands_pillar_roof_noise: DoublePerlinNoiseSampler,
    // Iceberg stuff
    iceberg_pillar_noise: DoublePerlinNoiseSampler,
    iceberg_pillar_roof_noise: DoublePerlinNoiseSampler,
    iceberg_surface_noise: DoublePerlinNoiseSampler,
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
            badlands_pillar_noise: noise_builder.get_noise_sampler_for_id("badlands_pillar"),
            badlands_surface_noise: noise_builder.get_noise_sampler_for_id("badlands_surface"),
            badlands_pillar_roof_noise: noise_builder
                .get_noise_sampler_for_id("badlands_pillar_roof"),
            iceberg_pillar_noise: noise_builder.get_noise_sampler_for_id("iceberg_pillar"),
            iceberg_pillar_roof_noise: noise_builder
                .get_noise_sampler_for_id("iceberg_pillar_roof"),
            iceberg_surface_noise: noise_builder.get_noise_sampler_for_id("iceberg_surface"),
        }
    }

    const ORANGE_TERRACOTTA: RawBlockState = default_block_state!("orange_terracotta");
    const YELLOW_TERRACOTTA: RawBlockState = default_block_state!("yellow_terracotta");
    const BROWN_TERRACOTTA: RawBlockState = default_block_state!("brown_terracotta");
    const RED_TERRACOTTA: RawBlockState = default_block_state!("red_terracotta");
    const WHITE_TERRACOTTA: RawBlockState = default_block_state!("white_terracotta");
    const LIGHT_GRAY_TERRACOTTA: RawBlockState = default_block_state!("light_gray_terracotta");
    const TERRACOTTA: RawBlockState = default_block_state!("terracotta");

    fn create_terracotta_bands(mut random: RandomGenerator) -> Box<[RawBlockState]> {
        let mut block_states = [Self::TERRACOTTA; 192];

        let mut i = 0;
        while i < block_states.len() {
            i += random.next_bounded_i32(5) as usize + 1;
            if i >= block_states.len() {
                break;
            }
            block_states[i] = Self::ORANGE_TERRACOTTA;
            i += 1;
        }

        Self::add_terracotta_bands(&mut random, &mut block_states, 1, Self::YELLOW_TERRACOTTA);
        Self::add_terracotta_bands(&mut random, &mut block_states, 2, Self::BROWN_TERRACOTTA);
        Self::add_terracotta_bands(&mut random, &mut block_states, 1, Self::RED_TERRACOTTA);

        let band_count = random.next_inbetween_i32(9, 15);
        let mut current_band = 0;
        let mut index = 0;

        while current_band < band_count && index < block_states.len() {
            block_states[index] = Self::WHITE_TERRACOTTA;

            if index > 1 && random.next_bool() {
                block_states[index - 1] = Self::LIGHT_GRAY_TERRACOTTA;
            }

            if index + 1 < block_states.len() && random.next_bool() {
                block_states[index + 1] = Self::LIGHT_GRAY_TERRACOTTA;
            }

            index += random.next_bounded_i32(16) as usize + 4;
            current_band += 1;
        }

        Box::new(block_states)
    }

    fn add_terracotta_bands(
        random: &mut RandomGenerator,
        terracotta_bands: &mut [RawBlockState],
        min_band_size: i32,
        state: RawBlockState,
    ) {
        let band_count = random.next_inbetween_i32(6, 15);

        for _ in 0..band_count {
            let band_width = min_band_size + random.next_bounded_i32(3);
            let start_index = random.next_bounded_i32(terracotta_bands.len() as i32);

            for m in 0..band_width {
                if (start_index + m < terracotta_bands.len() as i32) && (m < band_width) {
                    terracotta_bands[(start_index + m) as usize] = state;
                } else {
                    break; // Stop if we reach the end of the array
                }
            }
        }
    }

    pub fn place_badlands_pillar(
        &self,
        chunk: &mut ProtoChunk,
        global_x: i32,
        global_z: i32,
        surface_y: i32,
        default_state: RawBlockState,
    ) {
        let surface_noise =
            (self
                .badlands_surface_noise
                .sample(global_x as f64, 0.0, global_z as f64)
                * 8.25)
                .abs();
        let pillar_noise =
            self.badlands_pillar_noise
                .sample(global_x as f64 * 0.2, 0.0, global_z as f64 * 0.2)
                * 15.0;

        let threshold = surface_noise.min(pillar_noise);

        if threshold > 0.0 {
            let pillar_roof_noise = (self.badlands_pillar_roof_noise.sample(
                global_x as f64 * 0.75,
                0.0,
                global_z as f64 * 0.75,
            ) * 1.5)
                .abs();

            let scaled_threshold = threshold * threshold * 2.5;
            let transformed_roof = (pillar_roof_noise * 50.0).ceil() + 24.0;
            let elevation = 64.0 + scaled_threshold.min(transformed_roof);
            let elevation_y = elevation.floor() as i32;
            if surface_y <= elevation_y {
                for y in (chunk.bottom_y() as i32..=elevation_y).rev() {
                    let pos = Vector3::new(global_x, y, global_z);
                    let block_state = chunk.get_block_state(&pos).to_block();
                    if block_state == default_state.to_block() {
                        break;
                    }

                    if block_state == WATER_BLOCK.to_block() {
                        return;
                    }
                }

                for y in (chunk.bottom_y() as i32..=elevation_y).rev() {
                    let pos = Vector3::new(global_x, y, global_z);
                    let block_state = chunk.get_block_state(&pos).to_state();
                    if !block_state.is_air() {
                        break;
                    }

                    chunk.set_block_state(&pos, &default_state.to_state());
                }
            }
        }
    }

    const SNOW_BLOCK: RawBlockState = default_block_state!("snow_block");
    const PACKED_ICE: RawBlockState = default_block_state!("packed_ice");

    #[expect(clippy::too_many_arguments)]
    pub fn place_iceberg(
        &self,
        chunk: &mut ProtoChunk,
        biome: &Biome,
        x: i32,
        z: i32,
        estimated_surface_y: i32,
        current_top_y: i32,
        sea_level: i32,
        random_deriver: &RandomDeriver,
    ) {
        let iceburg_surface_noise =
            (self.iceberg_surface_noise.sample(x as f64, 0.0, z as f64) * 8.25).abs();

        let iceburg_pillar_noise =
            self.iceberg_pillar_noise
                .sample(x as f64 * 1.28, 0.0, z as f64 * 1.28)
                * 15.0;

        let threshold = iceburg_surface_noise.min(iceburg_pillar_noise);
        if threshold > 1.8 {
            let iceburg_pillar_roof_noise =
                (self
                    .iceberg_pillar_roof_noise
                    .sample(x as f64 * 1.17, 0.0, z as f64 * 1.17)
                    * 1.5)
                    .abs();

            let scaled_threshold = threshold * threshold * 1.2;
            let scaled_roof_noise = (iceburg_pillar_roof_noise * 40.0).ceil() + 14.0;

            let mut block_threshold = scaled_threshold.min(scaled_roof_noise);

            // TODO: Cache this
            let pos = Vector3::new(x, sea_level, z);
            let temperature = biome.weather.compute_temperature(&pos, sea_level);
            if temperature > 0.1f32 {
                block_threshold -= 2.0;
            }

            let (top_block, bottom_block) = if block_threshold > 2.0 {
                let value = sea_level as f64 - block_threshold - 7.0;
                (block_threshold as i32 + sea_level, value as i32)
            } else {
                (0, 0)
            };

            let mut rand = random_deriver.split_pos(x, 0, z);
            let snow_block_count = 2 + rand.next_bounded_i32(4);
            let snow_bottom = sea_level + 18 + rand.next_bounded_i32(10);
            let mut snow_blocks = 0;

            let top_y = current_top_y.max(top_block + 1);

            for y in (estimated_surface_y..=top_y).rev() {
                let pos = Vector3::new(x, y, z);
                let block_state = chunk.get_block_state(&pos);
                if (block_state.to_state().is_air() && y < top_block && rand.next_f64() > 0.01)
                    || (block_state.to_block() == WATER_BLOCK.to_block()
                        && y > bottom_block
                        && y < sea_level
                        && bottom_block != 0
                        && rand.next_f64() > 0.15)
                {
                    if snow_blocks <= snow_block_count && y > snow_bottom {
                        chunk.set_block_state(&pos, &Self::SNOW_BLOCK.to_state());
                        snow_blocks += 1;
                    } else {
                        chunk.set_block_state(&pos, &Self::PACKED_ICE.to_state());
                    }
                }
            }
        }
    }

    pub fn get_terracotta_block(&self, pos: &Vector3<i32>) -> BlockState {
        let offset = (self
            .terracotta_bands_offset_noise
            .sample(pos.x as f64, 0.0, pos.z as f64)
            * 4.0)
            .round() as i32;
        let offset = pos.y + offset;
        self.terracotta_bands
            [(offset as usize + self.terracotta_bands.len()) % self.terracotta_bands.len()]
        .to_state()
    }
}
