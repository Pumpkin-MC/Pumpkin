use serde::{Deserialize, Serialize};

#[must_use]
pub fn to_long(float: f32) -> i64 {
    (float * 10000f32) as i64
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct NoiseValuePoint {
    pub temperature: i64,
    pub humidity: i64,
    pub continentalness: i64,
    pub erosion: i64,
    pub depth: i64,
    pub weirdness: i64,
}

impl NoiseValuePoint {
    #[must_use]
    pub const fn convert_to_list(&self) -> [i64; 7] {
        [
            self.temperature,
            self.humidity,
            self.continentalness,
            self.erosion,
            self.depth,
            self.weirdness,
            0,
        ]
    }
}

#[cfg(test)]
mod test {
    use pumpkin_data::{
        chunk::Biome, chunk_gen_settings::GenerationSettings, dimension::Dimension,
        noise_router::OVERWORLD_BASE_NOISE_ROUTER,
    };
    use pumpkin_util::read_data_from_file;

    use crate::{
        GlobalRandomConfig, ProtoChunk,
        biome::{BiomeSupplier, MultiNoiseBiomeSupplier},
        block::to_state_from_blueprint,
        generation::{
            noise::router::{
                multi_noise_sampler::{MultiNoiseSampler, MultiNoiseSamplerBuilderOptions},
                proto_noise_router::ProtoNoiseRouters,
            },
            proto_chunk::TerrainCache,
        },
    };

    #[test]
    fn sample_value() {
        use crate::biome::hash_seed;
        use crate::generation::noise::router::multi_noise_sampler::{
            MultiNoiseSampler, MultiNoiseSamplerBuilderOptions,
        };
        use crate::generation::{biome_coords, positions::chunk_pos};
        type PosToPoint = (i32, i32, i32, i64, i64, i64, i64, i64, i64);
        let expected_data: Vec<PosToPoint> =
            read_data_from_file!("../../assets/multi_noise_sample_no_blend_no_beard_0_0_0.json");

        let seed = 0;
        let chunk_x = 0;
        let chunk_z = 0;

        let random_config = GlobalRandomConfig::new(seed, false);
        let noise_router =
            ProtoNoiseRouters::generate(&OVERWORLD_BASE_NOISE_ROUTER, &random_config);

        let surface_config = GenerationSettings::from_dimension(&Dimension::OVERWORLD);

        let _terrain_cache = TerrainCache::from_random(&random_config);
        // Calculate biome mixer seed
        let biome_mixer_seed = hash_seed(random_config.seed);

        let _chunk = ProtoChunk::new(
            chunk_x,
            chunk_z,
            &Dimension::OVERWORLD,
            to_state_from_blueprint(&surface_config.default_block),
            biome_mixer_seed,
        );

        let start_x = chunk_pos::start_block_x(chunk_x);
        let start_z = chunk_pos::start_block_z(chunk_z);
        let horizontal_biome_end = biome_coords::from_block(16);
        let multi_noise_config = MultiNoiseSamplerBuilderOptions::new(
            biome_coords::from_block(start_x),
            biome_coords::from_block(start_z),
            horizontal_biome_end as usize,
        );
        let mut multi_noise_sampler =
            MultiNoiseSampler::generate(&noise_router.multi_noise, &multi_noise_config);

        for (x, y, z, tem, hum, con, ero, dep, wei) in expected_data {
            let point = multi_noise_sampler.sample(x, y, z);
            assert_eq!(point.temperature, tem);
            assert_eq!(point.humidity, hum);
            assert_eq!(point.continentalness, con);
            assert_eq!(point.erosion, ero);
            assert_eq!(point.depth, dep);
            assert_eq!(point.weirdness, wei);
        }
    }

    #[test]
    fn sample_multinoise_biome() {
        let expected_data: Vec<(i32, i32, i32, u8)> =
            read_data_from_file!("../../assets/multi_noise_biome_source_test.json");

        let seed = 0;
        let random_config = GlobalRandomConfig::new(seed, false);
        let noise_router =
            ProtoNoiseRouters::generate(&OVERWORLD_BASE_NOISE_ROUTER, &random_config);

        let mut sampler = MultiNoiseSampler::generate(
            &noise_router.multi_noise,
            &MultiNoiseSamplerBuilderOptions::new(0, 0, 4),
        );

        for (x, y, z, biome_id) in expected_data {
            let calculated_biome = MultiNoiseBiomeSupplier::OVERWORLD.biome(x, y, z, &mut sampler);

            assert_eq!(
                biome_id,
                calculated_biome.id,
                "Expected {:?} was {:?} at {},{},{}",
                Biome::from_id(biome_id),
                calculated_biome,
                x,
                y,
                z
            );
        }
    }
}
