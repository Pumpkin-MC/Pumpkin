use sha2::{Digest, Sha256};
use std::{cell::RefCell, sync::LazyLock};

use enum_dispatch::enum_dispatch;
use multi_noise::BiomeTree;
use pumpkin_data::chunk::Biome;
use pumpkin_util::math::vector3::Vector3;

use crate::generation::noise_router::multi_noise_sampler::MultiNoiseSampler;
pub mod multi_noise;

pub static BIOME_SEARCH_TREE: LazyLock<BiomeTree> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../../../assets/multi_noise_biome_tree.json"))
        .expect("Could not parse multi_noise_biome_tree.json")
});

thread_local! {
    /// A shortcut; check if last used biome is what we should use
    static LAST_RESULT_NODE: RefCell<Option<&'static BiomeTree>> = const {RefCell::new(None) };
}

#[enum_dispatch]
pub trait BiomeSupplier {
    fn biome(at: &Vector3<i32>, noise: &mut MultiNoiseSampler<'_>) -> &'static Biome;
}

pub struct MultiNoiseBiomeSupplier;

// TODO: Add Nether & End supplier

impl BiomeSupplier for MultiNoiseBiomeSupplier {
    fn biome(global_biome_pos: &Vector3<i32>, noise: &mut MultiNoiseSampler<'_>) -> &'static Biome {
        let point = noise.sample(global_biome_pos.x, global_biome_pos.y, global_biome_pos.z);
        LAST_RESULT_NODE.with_borrow_mut(|last_result| BIOME_SEARCH_TREE.get(&point, last_result))
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
    use pumpkin_data::chunk::Biome;
    use pumpkin_util::math::{vector2::Vector2, vector3::Vector3};
    use serde::Deserialize;

    use crate::{
        GENERATION_SETTINGS, GeneratorSetting, GlobalProtoNoiseRouter, GlobalRandomConfig,
        NOISE_ROUTER_ASTS, ProtoChunk,
        generation::noise_router::multi_noise_sampler::{
            MultiNoiseSampler, MultiNoiseSamplerBuilderOptions,
        },
        read_data_from_file,
    };

    use super::{BiomeSupplier, MultiNoiseBiomeSupplier, hash_seed};

    #[test]
    fn test_biome_desert() {
        let seed = 13579;
        let random_config = GlobalRandomConfig::new(seed, false);
        let noise_rounter =
            GlobalProtoNoiseRouter::generate(&NOISE_ROUTER_ASTS.overworld, &random_config);
        let multi_noise_config = MultiNoiseSamplerBuilderOptions::new(1, 1, 1);
        let mut sampler = MultiNoiseSampler::generate(&noise_rounter, &multi_noise_config);
        let biome = MultiNoiseBiomeSupplier::biome(
            &pumpkin_util::math::vector3::Vector3 { x: -24, y: 1, z: 8 },
            &mut sampler,
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
            GlobalProtoNoiseRouter::generate(&NOISE_ROUTER_ASTS.overworld, &random_config);
        let surface_settings = GENERATION_SETTINGS
            .get(&GeneratorSetting::Overworld)
            .unwrap();

        for data in expected_data.into_iter() {
            let chunk_pos = Vector2::new(data.x, data.z);
            let mut chunk =
                ProtoChunk::new(chunk_pos, &noise_router, &random_config, surface_settings);
            chunk.populate_biomes();

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
}
