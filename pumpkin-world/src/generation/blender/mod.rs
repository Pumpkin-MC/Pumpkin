use std::sync::{Arc, LazyLock};

use enum_dispatch::enum_dispatch;

use super::noise::router::density_function::NoisePos;
use crate::generation::carver::CarvingStage;
use crate::generation::noise::perlin::DoublePerlinNoiseSampler;
use crate::generation::proto_chunk::{GenerationCache, ProtoChunk};
use crate::generation::settings::GenerationSettings;
use pumpkin_data::chunk::OFFSET;
use pumpkin_util::random::{RandomGenerator, xoroshiro128::Xoroshiro};

static SHIFT_NOISE: LazyLock<DoublePerlinNoiseSampler> = LazyLock::new(|| {
    let mut random = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(42));
    DoublePerlinNoiseSampler::from_params(&mut random, &OFFSET, false)
});

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

#[enum_dispatch]
pub trait BlenderImpl {
    fn calculate(&self, block_x: i32, block_z: i32) -> BlendResult;

    fn apply_blend_density(&self, pos: &impl NoisePos, density: f64) -> f64;

    fn get_biome_supplier(&self) {}
}

pub fn apply_carving_mask_filter<T: GenerationCache>(
    cache: &mut T,
    settings: &GenerationSettings,
) {
    if settings.debug_disable_blending {
        return;
    }

    if let Some(mask) = build_additional_mask(cache) {
        let chunk = cache.get_center_chunk_mut();
        chunk.set_additional_carving_mask(CarvingStage::Air, Arc::clone(&mask));
        chunk.set_additional_carving_mask(CarvingStage::Liquid, mask);
    }
}

fn build_additional_mask<T: GenerationCache>(
    cache: &T,
) -> Option<Arc<dyn Fn(i32, i32, i32) -> bool + Send + Sync + 'static>> {
    #[derive(Clone, Copy)]
    struct OldChunkBounds {
        center_y: f64,
        half_height: f64,
    }

    fn bounds_from(min_y: i32, height: i32) -> OldChunkBounds {
        let height = height as f64;
        let half_height = height / 2.0;
        OldChunkBounds {
            center_y: min_y as f64 + half_height,
            half_height,
        }
    }

    fn bounds_from_chunk(chunk: &ProtoChunk) -> Option<OldChunkBounds> {
        chunk
            .blending_data_old_generation_bounds()
            .map(|(min_y, height)| bounds_from(min_y, height))
    }

    let center = cache.get_center_chunk();
    let chunk_x = center.x;
    let chunk_z = center.z;

    let center_bounds = bounds_from_chunk(center);
    let west_bounds = cache
        .try_get_proto_chunk(chunk_x - 1, chunk_z)
        .and_then(bounds_from_chunk);
    let east_bounds = cache
        .try_get_proto_chunk(chunk_x + 1, chunk_z)
        .and_then(bounds_from_chunk);
    let north_bounds = cache
        .try_get_proto_chunk(chunk_x, chunk_z - 1)
        .and_then(bounds_from_chunk);
    let south_bounds = cache
        .try_get_proto_chunk(chunk_x, chunk_z + 1)
        .and_then(bounds_from_chunk);
    let northwest_bounds = cache
        .try_get_proto_chunk(chunk_x - 1, chunk_z - 1)
        .and_then(bounds_from_chunk);
    let northeast_bounds = cache
        .try_get_proto_chunk(chunk_x + 1, chunk_z - 1)
        .and_then(bounds_from_chunk);
    let southwest_bounds = cache
        .try_get_proto_chunk(chunk_x - 1, chunk_z + 1)
        .and_then(bounds_from_chunk);
    let southeast_bounds = cache
        .try_get_proto_chunk(chunk_x + 1, chunk_z + 1)
        .and_then(bounds_from_chunk);

    if center_bounds.is_none()
        && west_bounds.is_none()
        && east_bounds.is_none()
        && north_bounds.is_none()
        && south_bounds.is_none()
        && northwest_bounds.is_none()
        && northeast_bounds.is_none()
        && southwest_bounds.is_none()
        && southeast_bounds.is_none()
    {
        return None;
    }

    let bottom_y = cache.bottom_y() as i32;
    let top_y = cache.top_y() as i32;
    let candidates: [(f64, f64, Option<OldChunkBounds>); 9] = [
        (0.0, 0.0, center_bounds),
        (-16.0, 0.0, west_bounds),
        (16.0, 0.0, east_bounds),
        (0.0, -16.0, north_bounds),
        (0.0, 16.0, south_bounds),
        (-16.0, -16.0, northwest_bounds),
        (16.0, -16.0, northeast_bounds),
        (-16.0, 16.0, southwest_bounds),
        (16.0, 16.0, southeast_bounds),
    ];
    Some(Arc::new(move |offset_x, y, offset_z| {
        if y < bottom_y || y >= top_y {
            return false;
        }

        let shift_x = SHIFT_NOISE.sample(offset_x as f64, y as f64, offset_z as f64) * 4.0;
        let shift_y = SHIFT_NOISE.sample(y as f64, offset_z as f64, offset_x as f64) * 4.0;
        let shift_z = SHIFT_NOISE.sample(offset_z as f64, offset_x as f64, y as f64) * 4.0;
        let shifted_x = offset_x as f64 + 0.5 + shift_x;
        let shifted_y = y as f64 + 0.5 + shift_y;
        let shifted_z = offset_z as f64 + 0.5 + shift_z;

        let mut distance = f64::INFINITY;
        for (offset_x, offset_z, bounds) in candidates.iter() {
            let Some(bounds) = bounds else {
                continue;
            };
            distance = distance.min(distance_to_cube(
                shifted_x - 8.0 - offset_x,
                shifted_y - bounds.center_y,
                shifted_z - 8.0 - offset_z,
                8.0,
                bounds.half_height,
                8.0,
            ));
        }

        distance < 4.0
    }))
}

fn distance_to_cube(
    delta_x: f64,
    delta_y: f64,
    delta_z: f64,
    half_x: f64,
    half_y: f64,
    half_z: f64,
) -> f64 {
    let sx = (delta_x.abs() - half_x).max(0.0);
    let sy = (delta_y.abs() - half_y).max(0.0);
    let sz = (delta_z.abs() - half_z).max(0.0);
    (sx * sx + sy * sy + sz * sz).sqrt()
}

pub struct NoBlendBlender {}

impl BlenderImpl for NoBlendBlender {
    fn calculate(&self, _block_x: i32, _block_z: i32) -> BlendResult {
        BlendResult::new(1f64, 1f64)
    }

    fn apply_blend_density(&self, _pos: &impl NoisePos, density: f64) -> f64 {
        density
    }
}
