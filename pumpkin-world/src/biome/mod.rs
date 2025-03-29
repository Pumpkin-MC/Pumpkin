use sha2::{Digest, Sha256};
use std::{cell::RefCell, collections::HashMap, sync::LazyLock};

use enum_dispatch::enum_dispatch;
use multi_noise::{NoiseHypercube, SearchTree, TreeLeafNode};
use pumpkin_data::chunk::Biome;
use pumpkin_util::math::vector3::Vector3;
use serde::Deserialize;

use crate::{
    dimension::Dimension, generation::noise_router::multi_noise_sampler::MultiNoiseSampler,
};
pub mod multi_noise;

#[derive(Deserialize)]
pub struct BiomeEntries {
    biomes: Vec<BiomeEntry>,
}

#[derive(Deserialize)]
pub struct BiomeEntry {
    parameters: NoiseHypercube,
    biome: Biome,
}

pub static BIOME_ENTRIES: LazyLock<SearchTree<Biome>> = LazyLock::new(|| {
    let data: HashMap<Dimension, BiomeEntries> =
        serde_json::from_str(include_str!("../../../assets/multi_noise.json"))
            .expect("Could not parse multi_noise.json.");
    // TODO: support non overworld biomes
    let overworld_data = data
        .get(&Dimension::Overworld)
        .expect("Overworld dimension not found");

    let entries: Vec<(Biome, &NoiseHypercube)> = overworld_data
        .biomes
        .iter()
        .map(|entry| (entry.biome, &entry.parameters))
        .collect();

    SearchTree::create(entries)
});

thread_local! {
    static LAST_RESULT_NODE: RefCell<Option<TreeLeafNode<Biome>>> = const {RefCell::new(None) };
}

#[enum_dispatch]
pub trait BiomeSupplier {
    fn biome(at: &Vector3<i32>, noise: &mut MultiNoiseSampler<'_>) -> Biome;
}

pub struct MultiNoiseBiomeSupplier;

// TODO: Add End supplier

impl BiomeSupplier for MultiNoiseBiomeSupplier {
    fn biome(at: &Vector3<i32>, noise: &mut MultiNoiseSampler<'_>) -> Biome {
        //panic!("{}:{}:{}", at.x, at.y, at.z);
        let point = noise.sample(at.x, at.y, at.z);
        LAST_RESULT_NODE.with_borrow_mut(|last_result| {
            BIOME_ENTRIES
                .get(&point, last_result)
                .expect("failed to get biome entry")
        })
    }
}

pub fn hash_seed(seed: i64) -> i64 {
    let mut hasher = Sha256::new();
    hasher.update(seed.to_be_bytes());
    let result = hasher.finalize();
    i64::from_be_bytes(result[..8].try_into().unwrap())
}

#[cfg(test)]
mod test {
    use pumpkin_data::chunk::Biome;

    use crate::{
        GlobalProtoNoiseRouter, GlobalRandomConfig, NOISE_ROUTER_ASTS,
        generation::noise_router::multi_noise_sampler::{
            MultiNoiseSampler, MultiNoiseSamplerBuilderOptions,
        },
    };

    use super::{BiomeSupplier, MultiNoiseBiomeSupplier};

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
        assert_eq!(biome, Biome::DESERT)
    }
}
