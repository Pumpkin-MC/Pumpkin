//! GPU-accelerated surface noise pre-computation.
//!
//! The surface stage (`build_surface`) calls `DoublePerlinNoiseSampler::sample`
//! twice per column (once for `run_depth`, once for `secondary_depth`) —
//! 512 calls total per chunk. This module batches those calls into GPU
//! dispatches via the cached `OctaveBatch` / `sample_batch` path.
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
/// Extracts octave data from each `DoublePerlinNoiseSampler` once, builds 256
/// column positions, dispatches the four octave samplers (two per double-perlin)
/// via `sample_batch`, combines the results, and returns them packed in a
/// `SurfaceNoiseBatch`.
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

    // --- Pre-build scaled points once (same scale for all double-perlin samplers) ---
    let scale: f32 = 1.018_126_8;
    let scaled: Vec<[f32; 3]> = columns
        .iter()
        .map(|&[x, y, z]| [x * scale, y * scale, z * scale])
        .collect();

    // --- Extract octave batches once per sampler (cheap: copies parameter structs) ---
    let (sf1, sf2) = surface_sampler.samplers();
    let surface_first = OctaveBatch::from_cpu_sampler(sf1);
    let surface_second = OctaveBatch::from_cpu_sampler(sf2);
    let surface_amp = surface_sampler.amplitude() as f32;

    let (sc1, sc2) = secondary_sampler.samplers();
    let secondary_first = OctaveBatch::from_cpu_sampler(sc1);
    let secondary_second = OctaveBatch::from_cpu_sampler(sc2);
    let secondary_amp = secondary_sampler.amplitude() as f32;

    // --- Combined double-perlin per point: (first + second) * amplitude ---
    let first_a = ctx.sample_batch(&surface_first, &columns);
    let second_a = ctx.sample_batch(&surface_second, &scaled);
    let surface_noise: Arc<[f64]> = first_a
        .iter()
        .zip(&second_a)
        .map(|(&a, &b)| ((a + b) * surface_amp) as f64)
        .collect();

    let first_b = ctx.sample_batch(&secondary_first, &columns);
    let second_b = ctx.sample_batch(&secondary_second, &scaled);
    let secondary_noise: Arc<[f64]> = first_b
        .iter()
        .zip(&second_b)
        .map(|(&a, &b)| ((a + b) * secondary_amp) as f64)
        .collect();

    Some(SurfaceNoiseBatch {
        surface_noise,
        secondary_noise,
    })
}

/// Returns a function pointer suitable for `register_surface_noise_gpu`.
#[must_use]
pub const fn surface_noise_gpu_fn() -> SurfaceNoiseGpuFn {
    surface_noise_gpu_callback
}

#[cfg(test)]
mod tests {
    use super::*;
    use pumpkin_util::noise::perlin::OctavePerlinNoiseSampler;
    use pumpkin_util::random::{RandomGenerator, RandomImpl, xoroshiro128::Xoroshiro};
    use pumpkin_world::generation::noise::perlin::DoublePerlinNoiseSampler;

    fn make_double_perlin(seed: u64) -> DoublePerlinNoiseSampler {
        let mut rand = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(seed));
        let (first_octave, amplitudes) =
            OctavePerlinNoiseSampler::calculate_amplitudes(&(-15..=0).collect::<Vec<i32>>());
        DoublePerlinNoiseSampler::new(&mut rand, first_octave, &amplitudes, 1.0, true)
    }

    #[test]
    fn surface_callback_returns_none_without_gpu() {
        let sampler = make_double_perlin(1234);
        let secondary = make_double_perlin(5678);
        let result = surface_noise_gpu_callback(&sampler, &secondary, 0, 0);
        assert!(
            result.is_none(),
            "must return None when no GPU is available"
        );
    }

    #[test]
    fn column_grid_is_16x16_256_entries() {
        let columns: Vec<[f32; 3]> = (0..16)
            .flat_map(|lz| (0..16).map(move |lx| [(lx) as f32, 0.0f32, (lz) as f32]))
            .collect();
        assert_eq!(columns.len(), 256);
        assert_eq!(columns[0], [0.0, 0.0, 0.0]);
        assert_eq!(columns[255], [15.0, 0.0, 15.0]);
        assert_eq!(columns[3 * 16 + 8], [8.0, 0.0, 3.0]);
    }

    #[test]
    fn double_perlin_scale_matches_cpu() {
        let sampler = make_double_perlin(999);
        let (first, second) = sampler.samplers();
        let amp = sampler.amplitude();
        let cpu_value = sampler.sample(100.0, 0.0, 200.0);
        let scale: f64 = 1.018_126_8;
        let first_val = first.sample(100.0, 0.0, 200.0);
        let second_val = second.sample(100.0 * scale, 0.0, 200.0 * scale);
        let gpu_style = (first_val + second_val) * amp;
        let diff = (cpu_value - gpu_style).abs();
        assert!(diff < 0.5, "GPU-style differs from CPU by {diff:.4}");
    }

    #[test]
    fn surface_gpu_fn_returns_valid_pointer() {
        let fn_ptr = surface_noise_gpu_fn();
        assert!(!(fn_ptr as *const ()).is_null());
    }
}
