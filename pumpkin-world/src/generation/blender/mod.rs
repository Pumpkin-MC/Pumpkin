use enum_dispatch::enum_dispatch;
use pumpkin_data::chunk::Biome;
use pumpkin_util::math::vector3::Vector3;

use crate::{
    biome::BiomeSupplier, generation::noise::router::multi_noise_sampler::MultiNoiseSampler,
};

pub struct BlendResult {
    alpha: f64,
    offset: f64,
}

impl BlendResult {
    pub const fn new(alpha: f64, offset: f64) -> Self {
        Self { alpha, offset }
    }
}

#[enum_dispatch(BlenderImpl)]
pub enum Blender {
    NoBlend(NoBlendBlender),
}

impl Blender {
    pub const NO_BLEND: Self = Self::NoBlend(NoBlendBlender {});
}

pub struct BlenderBiomeSupplier<'a> {
    base: &'a dyn BiomeSupplier,
}

impl BiomeSupplier for BlenderBiomeSupplier<'_> {
    fn biome(&self, x: i32, y: i32, z: i32, sampler: &mut MultiNoiseSampler<'_>) -> &'static Biome {
        self.base.biome(x, y, z, sampler)
    }
}

#[enum_dispatch]
pub trait BlenderImpl {
    fn calculate(&self, block_x: i32, block_z: i32) -> BlendResult;

    fn apply_blend_density(&self, pos: &Vector3<i32>, density: f64) -> f64;

    fn get_biome_supplier<'a>(&self, supplier: &'a dyn BiomeSupplier) -> BlenderBiomeSupplier<'a>;
}

pub struct NoBlendBlender {}

impl BlenderImpl for NoBlendBlender {
    fn calculate(&self, _block_x: i32, _block_z: i32) -> BlendResult {
        BlendResult::new(1f64, 1f64)
    }

    fn apply_blend_density(&self, _pos: &Vector3<i32>, density: f64) -> f64 {
        density
    }

    fn get_biome_supplier<'a>(&self, supplier: &'a dyn BiomeSupplier) -> BlenderBiomeSupplier<'a> {
        BlenderBiomeSupplier { base: supplier }
    }
}
