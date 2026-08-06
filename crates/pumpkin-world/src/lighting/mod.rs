pub mod engine;
pub mod storage;

pub use engine::LightEngine;

pub mod runtime;
pub use runtime::DynamicLightEngine;

// ---------------------------------------------------------------------------
// GPU acceleration hook
// ---------------------------------------------------------------------------
//
// pumpkin-world does NOT depend on pumpkin-gpu.  Instead it exposes a
// function-pointer slot that the main server crate (pumpkin) can fill at
// startup.  When a GPU callback is registered, the sky-light scanner in
// `engine.rs` calls it before falling back to the CPU path.

use std::sync::OnceLock;

/// Signature for a GPU-accelerated sky-light column scanner.
///
/// Arguments:
/// - `opacity`: per-block opacity values, flattened `[column][y]` (row-major over
///   the 18×18 column grid, then Y within each column).
/// - `heightmap`: the top solid Y per column (`num_columns` entries).
/// - `num_columns`, `height`, `bottom_y`: region dimensions (see [`super::light::SkyLightInput`]).
///
/// Returns `Some(light_values)` where `light_values` is the same layout as `opacity`
/// (one `u8` per block position), or `None` when the GPU path is unavailable.
pub type SkyLightGpuFn = fn(
    opacity: &[u8],
    heightmap: &[i32],
    num_columns: u32,
    height: u32,
    bottom_y: i32,
) -> Option<Vec<u8>>;

static SKY_LIGHT_GPU: OnceLock<SkyLightGpuFn> = OnceLock::new();

/// Register a GPU sky-light scanner.  Call once at server startup.
/// Subsequent calls are no-ops.
pub fn register_sky_light_gpu(f: SkyLightGpuFn) {
    let _ = SKY_LIGHT_GPU.set(f);
}

/// Returns the registered GPU sky-light scanner, if any.
#[must_use]
pub fn get_sky_light_gpu() -> Option<SkyLightGpuFn> {
    SKY_LIGHT_GPU.get().copied()
}
