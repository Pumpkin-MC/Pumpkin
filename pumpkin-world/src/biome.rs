use enum_dispatch::enum_dispatch;
use pumpkin_data::chunk::Biome;

#[derive(Clone)]
#[enum_dispatch(BiomeSupplierImpl)]
pub enum BiomeSupplier {
    Debug(DebugBiomeSupplier),
}

#[enum_dispatch]
pub trait BiomeSupplierImpl {
    fn biome(&self, x: i32, y: i32, z: i32, noise: &MultiNoiseSampler) -> Biome;
}

#[derive(Clone)]
pub struct DebugBiomeSupplier {}

impl BiomeSupplierImpl for DebugBiomeSupplier {
    fn biome(&self, _x: i32, _y: i32, _z: i32, _noise: &MultiNoiseSampler) -> Biome {
        Biome::Plains
    }
}

// TODO: Implement
pub struct MultiNoiseSampler {}
