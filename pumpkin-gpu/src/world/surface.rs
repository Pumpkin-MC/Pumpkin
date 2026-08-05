//! GPU-accelerated surface noise pre-computation.
//!
//! The surface stage (`build_surface`) calls `DoublePerlinNoiseSampler::sample`
//! twice per column (once for `run_depth`, once for `secondary_depth`) —
//! 512 calls total per chunk. This module batches those calls into two GPU
//! dispatches via the existing `OctaveBatch` / `sample_batch` path.
//!
//! Registered as a callback in `pumpkin_world::generation::surface`, following
//! the same `OnceLock<fn>` pattern as `pumpkin_world::lighting::register_sky_light_gpu`.

use std::sync::Arc;

use pumpkin_world::generation::{
    noise::perlin::DoublePerlinNoiseSampler,
    surface::{SurfaceNoiseBatch, SurfaceNoiseGpuFn},
};

use crate::world::{gpu::OctaveBatch, light::get_global_gpu};

/// GPU callback for surface noise batch pre-computation.
///
/// Extracts octave data from each `DoublePerlinNoiseSampler`, builds column
/// positions for the 16×16 chunk, dispatches the two component octave samplers
/// for each `DoublePerlinNoiseSampler`, combines the results, and returns them
/// packed in a `SurfaceNoiseBatch`.
///
/// Returns `None` when no global GPU context is available (caller falls back to
/// per-column CPU sampling).
#[must_use]
pub fn surface_noise_gpu_callback(
    surface_sampler: &DoublePerlinNoiseSampler,
    secondary_sampler: &DoublePerlinNoiseSampler,
    start_x: i32,
    start_z: i32,
) -> Option<SurfaceNoiseBatch> {
    let ctx = get_global_gpu()?;

    // --- Build column positions (16×16 grid, y=0) ---
    let columns: Vec<[f32; 3]> = (0..16)
        .flat_map(|lz| {
            (0..16).map(move |lx| [(start_x + lx) as f32, 0.0f32, (start_z + lz) as f32])
        })
        .collect();

    // --- Surface noise: combine two octave samplers ---
    let surface_values = batch_double_perlin(ctx, surface_sampler, &columns);

    // --- Secondary noise: combine two octave samplers ---
    let secondary_values = batch_double_perlin(ctx, secondary_sampler, &columns);

    // Convert f32 GPU output to f64 (the surface code uses f64).
    #[expect(clippy::cast_lossless)]
    let surface_noise: Arc<[f64]> = surface_values.iter().map(|&v| v as f64).collect();
    #[expect(clippy::cast_lossless)]
    let secondary_noise: Arc<[f64]> = secondary_values.iter().map(|&v| v as f64).collect();

    Some(SurfaceNoiseBatch {
        surface_noise,
        secondary_noise,
    })
}

/// Evaluate a `DoublePerlinNoiseSampler` on a batch of points.
///
/// A double-perlin sampler combines two octave samplers with a scaling factor:
/// `(first.sample(x,y,z) + second.sample(x*1.018..., y*1.018..., z*1.018...)) * amplitude`.
///
/// This function dispatches both octave samplers on the GPU and combines the results.
fn batch_double_perlin(
    ctx: &crate::GpuNoiseContext,
    sampler: &DoublePerlinNoiseSampler,
    points: &[[f32; 3]],
) -> Vec<f32> {
    let (first_sampler, second_sampler) = sampler.samplers();
    let amplitude = sampler.amplitude() as f32;

    // Build scaled points for the second sampler.
    let scale: f32 = 1.018_126_8;
    let scaled_points: Vec<[f32; 3]> = points
        .iter()
        .map(|&[x, y, z]| [x * scale, y * scale, z * scale])
        .collect();

    let first_batch = OctaveBatch::from_cpu_sampler(first_sampler);
    let second_batch = OctaveBatch::from_cpu_sampler(second_sampler);

    let first_values = ctx.sample_batch(&first_batch, points);
    let second_values = ctx.sample_batch(&second_batch, &scaled_points);

    first_values
        .iter()
        .zip(&second_values)
        .map(|(&a, &b)| (a + b) * amplitude)
        .collect()
}

/// Returns a function pointer suitable for `register_surface_noise_gpu`.
#[must_use]
pub const fn surface_noise_gpu_fn() -> SurfaceNoiseGpuFn {
    surface_noise_gpu_callback
}
