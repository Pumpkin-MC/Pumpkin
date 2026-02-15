use enum_dispatch::enum_dispatch;
use pumpkin_data::{chunk::Biome, dimension::Dimension};
use pumpkin_util::math::vector3::Vector3;

use crate::{
    biome::{BiomeSupplier, MultiNoiseBiomeSupplier, end::TheEndBiomeSupplier},
    generation::noise::router::multi_noise_sampler::MultiNoiseSampler,
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

pub type BlenderBiomeSupplier = for<'a> fn(
    x: i32,
    y: i32,
    z: i32,
    sampler: &mut MultiNoiseSampler<'a>,
    dimension: Dimension,
) -> &'static Biome;

#[enum_dispatch]
pub trait BlenderImpl {
    fn calculate(&self, block_x: i32, block_z: i32) -> BlendResult;

    fn apply_blend_density(&self, pos: &Vector3<i32>, density: f64) -> f64;

    fn get_biome_supplier(&self) -> BlenderBiomeSupplier;
}

pub struct NoBlendBlender {}

impl BlenderImpl for NoBlendBlender {
    fn calculate(&self, _block_x: i32, _block_z: i32) -> BlendResult {
        BlendResult::new(1f64, 1f64)
    }

    fn apply_blend_density(&self, _pos: &Vector3<i32>, density: f64) -> f64 {
        density
    }

    fn get_biome_supplier(&self) -> BlenderBiomeSupplier {
        no_blend_biome_supplier
    }
}

fn no_blend_biome_supplier(
    x: i32,
    y: i32,
    z: i32,
    sampler: &mut MultiNoiseSampler<'_>,
    dimension: Dimension,
) -> &'static Biome {
    if dimension == Dimension::THE_END {
        TheEndBiomeSupplier::biome(x, y, z, sampler, dimension)
    } else {
        MultiNoiseBiomeSupplier::biome(x, y, z, sampler, dimension)
    }
}
