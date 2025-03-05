use std::{cell::RefCell, sync::LazyLock};

use enum_dispatch::enum_dispatch;
use multi_noise::{BiomeEntries, SearchTree, TreeLeafNode};
use pumpkin_data::chunk::Biome;

use crate::{
    coordinates::BlockCoordinates, generation::noise_router::multi_noise_sampler::MultiNoiseSampler,
};
pub mod multi_noise;

pub static BIOME_ENTRIES: LazyLock<SearchTree<Biome>> = LazyLock::new(|| {
    SearchTree::create(
        serde_json::from_str::<BiomeEntries>(include_str!("../../../assets/multi_noise.json"))
            .expect("Could not parse multi_noise.json.")
            .nodes
            .into_iter()
            .flat_map(|(_, biome_map)| biome_map.into_iter())
            .collect(),
    )
    .expect("entries cannot be empty")
});

thread_local! {
    static LAST_RESULT_NODE: RefCell<Option<TreeLeafNode<Biome>>> = const {RefCell::new(None) };
}

#[enum_dispatch]
pub trait BiomeSupplier {
    fn biome(&mut self, at: BlockCoordinates) -> Biome;
}

#[derive(Clone)]
pub struct DebugBiomeSupplier;

impl BiomeSupplier for DebugBiomeSupplier {
    fn biome(&mut self, _at: BlockCoordinates) -> Biome {
        Biome::Plains
    }
}

pub struct MultiNoiseBiomeSupplier<'a> {
    noise: MultiNoiseSampler<'a>,
}

impl BiomeSupplier for MultiNoiseBiomeSupplier<'_> {
    fn biome(&mut self, at: BlockCoordinates) -> Biome {
        let point = self.noise.sample(at.x, at.y.0 as i32, at.z);
        LAST_RESULT_NODE.with_borrow_mut(|last_result| {
            BIOME_ENTRIES
                .get(&point, last_result)
                .expect("failed to get biome entry")
        })
    }
}
