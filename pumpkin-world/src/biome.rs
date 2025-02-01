use std::sync::LazyLock;

use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};
use serde_with::{DeserializeFromStr, SerializeDisplay};

use crate::{
    coordinates::BlockCoordinates,
    generation::{
        multi_noise_sampler::{BiomeEntries, MultiNoiseSampler},
        noise::density::{NoisePos, UnblendedNoisePos},
        GeneratorInit, Seed,
    },
};

use pumpkin_data::chunk::Biome;

pub static BIOME_ENTRIES: LazyLock<BiomeEntries> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../../assets/multi_noise.json"))
        .expect("Could not parse synced_registries.json registry.")
});

pub trait BiomeSupplier {
    fn biome(&self, at: BlockCoordinates) -> Biome;
}

#[derive(Clone)]
pub struct DebugBiomeSupplier;

impl BiomeSupplier for DebugBiomeSupplier {
    fn biome(&self, _at: BlockCoordinates) -> Biome {
        Biome::Plains
    }
}

#[derive(Clone)]
pub struct MultiNoiseBiomeSupplier{noise: MultiNoiseSampler};

impl BiomeSupplier for MultiNoiseBiomeSupplier {
    fn biome(&self, at: BlockCoordinates) -> Biome {
        BIOME_ENTRIES.find_biome(&noise.sample(&NoisePos::Unblended(UnblendedNoisePos::new(
            at.x,
            at.y.0 as i32,
            at.z,
        ))))
    }
}

pub(crate) struct SuperflatBiomeGenerator {}

impl GeneratorInit for SuperflatBiomeGenerator {
    fn new(_: Seed) -> Self {
        Self {}
    }
}
