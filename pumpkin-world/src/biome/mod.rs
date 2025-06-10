use sha2::{Digest, Sha256};
use std::cell::RefCell;

use enum_dispatch::enum_dispatch;
use pumpkin_data::chunk::{Biome, BiomeTree, NETHER_BIOME_SOURCE, OVERWORLD_BIOME_SOURCE};
use pumpkin_util::math::vector3::Vector3;

use crate::{
    dimension::Dimension, generation::noise_router::multi_noise_sampler::MultiNoiseSampler,
};
pub mod end;
pub mod multi_noise;

thread_local! {
    /// A shortcut; check if last used biome is what we should use
    static LAST_RESULT_NODE: RefCell<Option<&'static BiomeTree>> = const {RefCell::new(None) };
}

#[enum_dispatch]
pub trait BiomeSupplier {
    fn biome(
        at: &Vector3<i32>,
        noise: &mut MultiNoiseSampler<'_>,
        dimension: Dimension,
    ) -> &'static Biome;
}

pub struct MultiNoiseBiomeSupplier;

impl BiomeSupplier for MultiNoiseBiomeSupplier {
    fn biome(
        global_biome_pos: &Vector3<i32>,
        noise: &mut MultiNoiseSampler<'_>,
        dimension: Dimension,
    ) -> &'static Biome {
        let source: &'static BiomeTree = match dimension {
            Dimension::Overworld => &OVERWORLD_BIOME_SOURCE,
            Dimension::Nether => &NETHER_BIOME_SOURCE,
            Dimension::End => unreachable!(), // Use TheEndBiomeSupplier
        };
        let point = noise.sample(global_biome_pos.x, global_biome_pos.y, global_biome_pos.z);
        let point_list = point.convert_to_list();
        LAST_RESULT_NODE.with_borrow_mut(|last_result| source.get(&point_list, last_result))
    }
}

pub fn hash_seed(seed: u64) -> i64 {
    let mut hasher = Sha256::new();
    hasher.update(seed.to_le_bytes());
    let result = hasher.finalize();
    i64::from_le_bytes(result[..8].try_into().unwrap())
}

#[cfg(test)]
mod test {
    use pumpkin_data::{chunk::Biome, noise_router::OVERWORLD_BASE_NOISE_ROUTER};
    use pumpkin_util::math::{vector2::Vector2, vector3::Vector3};
    use serde::Deserialize;

    use crate::{
        GENERATION_SETTINGS, GeneratorSetting, GlobalRandomConfig, ProtoChunk,
        chunk::palette::BIOME_NETWORK_MAX_BITS,
        dimension::Dimension,
        generation::noise_router::{
            multi_noise_sampler::{MultiNoiseSampler, MultiNoiseSamplerBuilderOptions},
            proto_noise_router::ProtoNoiseRouters,
        },
        read_data_from_file,
    };

    use super::{BiomeSupplier, MultiNoiseBiomeSupplier, hash_seed};

    #[test]
    fn test_biome_desert() {
        let seed = 13579;
        let random_config = GlobalRandomConfig::new(seed, false);
        let noise_router =
            ProtoNoiseRouters::generate(&OVERWORLD_BASE_NOISE_ROUTER, &random_config);
        let multi_noise_config = MultiNoiseSamplerBuilderOptions::new(1, 1, 1);
        let mut sampler =
            MultiNoiseSampler::generate(&noise_router.multi_noise, &multi_noise_config);
        let biome = MultiNoiseBiomeSupplier::biome(
            &pumpkin_util::math::vector3::Vector3 { x: -24, y: 1, z: 8 },
            &mut sampler,
            Dimension::Overworld,
        );
        assert_eq!(biome, &Biome::DESERT)
    }

    #[test]
    fn test_wide_area_surface() {
        #[derive(Deserialize)]
        struct BiomeData {
            x: i32,
            z: i32,
            data: Vec<(i32, i32, i32, u8)>,
        }

        let expected_data: Vec<BiomeData> =
            read_data_from_file!("../../assets/biome_no_blend_no_beard_0.json");

        let seed = 0;
        let random_config = GlobalRandomConfig::new(seed, false);
        let noise_router =
            ProtoNoiseRouters::generate(&OVERWORLD_BASE_NOISE_ROUTER, &random_config);
        let surface_settings = GENERATION_SETTINGS
            .get(&GeneratorSetting::Overworld)
            .unwrap();

        for data in expected_data.into_iter() {
            let chunk_pos = Vector2::new(data.x, data.z);
            let mut chunk =
                ProtoChunk::new(chunk_pos, &noise_router, &random_config, surface_settings);
            chunk.populate_biomes(Dimension::Overworld);

            for (biome_x, biome_y, biome_z, biome_id) in data.data {
                let global_biome_pos = Vector3::new(biome_x, biome_y, biome_z);
                let calculated_biome = chunk.get_biome(&global_biome_pos);

                assert_eq!(
                    biome_id,
                    calculated_biome.id,
                    "Expected {:?} was {:?} at {},{},{} ({},{})",
                    Biome::from_id(biome_id),
                    calculated_biome,
                    biome_x,
                    biome_y,
                    biome_z,
                    data.x,
                    data.z
                );
            }
        }
    }

    #[test]
    fn test_hash_seed() {
        let hashed_seed = hash_seed(0);
        assert_eq!(8794265229978523055, hashed_seed);

        let hashed_seed = hash_seed((-777i64) as u64);
        assert_eq!(-1087248400229165450, hashed_seed);
    }

    #[test]
    fn test_proper_network_bits_per_entry() {
        let id_to_test = 1 << BIOME_NETWORK_MAX_BITS;
        if Biome::from_id(id_to_test).is_some() {
            panic!("We need to update our constants!");
        }
    }
}
