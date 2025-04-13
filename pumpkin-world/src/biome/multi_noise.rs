use serde::{Deserialize, Serialize};

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
    pub fn convert_to_list(&self) -> [i64; 7] {
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
    use pumpkin_data::{chunk::Biome, noise_router::OVERWORLD_BASE_NOISE_ROUTER};
    use pumpkin_util::math::{vector2::Vector2, vector3::Vector3};

    use crate::{
        GENERATION_SETTINGS, GeneratorSetting, GlobalRandomConfig, ProtoChunk,
        biome::{BiomeSupplier, MultiNoiseBiomeSupplier},
        generation::noise_router::{
            multi_noise_sampler::{MultiNoiseSampler, MultiNoiseSamplerBuilderOptions},
            proto_noise_router::ProtoNoiseRouters,
        },
        read_data_from_file,
    };

    #[test]
    fn test_sample_value() {
        type PosToPoint = (i32, i32, i32, i64, i64, i64, i64, i64, i64);
        let expected_data: Vec<PosToPoint> =
            read_data_from_file!("../../assets/multi_noise_sample_no_blend_no_beard_0_0_0.json");

        let seed = 0;
        let chunk_pos = Vector2::new(0, 0);
        let random_config = GlobalRandomConfig::new(seed, false);
        let noise_rounter =
            ProtoNoiseRouters::generate(&OVERWORLD_BASE_NOISE_ROUTER, &random_config);

        let surface_config = GENERATION_SETTINGS
            .get(&GeneratorSetting::Overworld)
            .unwrap();

        let mut chunk = ProtoChunk::new(chunk_pos, &noise_rounter, &random_config, surface_config);

        for (x, y, z, tem, hum, con, ero, dep, wei) in expected_data.into_iter() {
            let point = chunk.multi_noise_sampler.sample(x, y, z);
            assert_eq!(point.temperature, tem);
            assert_eq!(point.humidity, hum);
            assert_eq!(point.continentalness, con);
            assert_eq!(point.erosion, ero);
            assert_eq!(point.depth, dep);
            assert_eq!(point.weirdness, wei);
        }
    }

    #[test]
    fn test_sample_multinoise_biome() {
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
            let global_biome_pos = Vector3::new(x, y, z);
            let calculated_biome = MultiNoiseBiomeSupplier::biome(&global_biome_pos, &mut sampler);

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
