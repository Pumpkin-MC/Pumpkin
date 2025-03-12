use std::{cell::RefCell, collections::HashMap, sync::LazyLock};

use enum_dispatch::enum_dispatch;
use multi_noise::{NoiseHypercube, SearchTree, TreeLeafNode};
use pumpkin_data::chunk::Biome;
use pumpkin_util::math::vector3::Vector3;

use crate::{
    dimension::Dimension, generation::noise_router::multi_noise_sampler::MultiNoiseSampler,
};
pub mod multi_noise;

pub static BIOME_ENTRIES: LazyLock<SearchTree<Biome>> = LazyLock::new(|| {
    let data: HashMap<Dimension, HashMap<Biome, NoiseHypercube>> =
        serde_json::from_str(include_str!("../../../assets/multi_noise.json"))
            .expect("Could not parse multi_noise.json.");
    // TODO: support non overworld biomes
    let overworld_data = data
        .get(&Dimension::Overworld)
        .expect("Overworld dimension not found");

    let entries: Vec<(Biome, NoiseHypercube)> = overworld_data
        .iter()
        .map(|(biome, biome_map)| (*biome, biome_map.clone()))
        .collect();

    SearchTree::create(entries).expect("entries cannot be empty")
});

thread_local! {
    static LAST_RESULT_NODE: RefCell<Option<TreeLeafNode<Biome>>> = const {RefCell::new(None) };
}

#[enum_dispatch]
pub trait BiomeSupplier {
    fn biome(at: &Vector3<i32>, noise: &mut MultiNoiseSampler<'_>) -> Biome;
}

#[derive(Clone)]
pub struct DebugBiomeSupplier;

impl BiomeSupplier for DebugBiomeSupplier {
    fn biome(_at: &Vector3<i32>, _noise: &mut MultiNoiseSampler<'_>) -> Biome {
        Biome::Plains
    }
}

pub struct MultiNoiseBiomeSupplier;

// TODO: Add End supplier

impl BiomeSupplier for MultiNoiseBiomeSupplier {
    fn biome(at: &Vector3<i32>, noise: &mut MultiNoiseSampler<'_>) -> Biome {
        let point = noise.sample(at.x, at.y, at.z);
        LAST_RESULT_NODE.with_borrow_mut(|last_result| {
            BIOME_ENTRIES
                .get(&point, last_result)
                .expect("failed to get biome entry")
        })
    }
}
