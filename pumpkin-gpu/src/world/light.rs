//! GPU-side types and helpers for initial light scanning.
//!
//! The WGSL compute shader lives in `light.wgsl` alongside this file.
//! Dispatch is handled by [`super::gpu::GpuNoiseContext`].
//!
//! # Global GPU context
//!
//! Call [`init_global_gpu`] once at server startup. After that,
//! [`try_sky_light_gpu`] is safe to call from any thread and returns
//! computed sky-light values when a GPU is available, or `None` to
//! signal "fall back to the CPU path."

use super::gpu::GpuNoiseContext;
use bytemuck::{Pod, Zeroable};
use std::sync::OnceLock;

static GLOBAL_GPU: OnceLock<GpuNoiseContext> = OnceLock::new();

/// Backend/adapter selection hint shared between [`GpuNoiseContext`] and this
/// module.  Not public API — used internally for config-driven adapter selection.
#[derive(Debug, Clone, Copy)]
pub enum AdapterSelector {
    /// Let wgpu auto-detect the backend.
    Auto,
    /// Force a specific `wgpu::Backend`.
    Specific(wgpu::Backend),
}

/// Initialize the global GPU context with auto-detected adapter.
///
/// Safe to call multiple times; subsequent calls are no-ops. When no
/// compatible GPU is available the global slot stays empty — every
/// [`try_sky_light_gpu`] / [`sky_light_gpu_callback`] call will return
/// `None` and the caller falls back to the CPU path.
pub fn init_global_gpu() {
    if GLOBAL_GPU.get().is_none()
        && let Some(ctx) = GpuNoiseContext::try_new()
    {
        let _ = GLOBAL_GPU.set(ctx);
    }
}

/// Initialize the global GPU context with explicit device/backend selection.
///
/// Uses the user's configuration for adapter choice, backend forcing, and
/// device filtering. Safe to call multiple times; subsequent calls are
/// no-ops. When `config.enabled` is `false` this is a no-op.
pub fn init_global_gpu_with_config(config: &pumpkin_config::gpu::GpuConfig) {
    if !config.enabled {
        return;
    }
    if GLOBAL_GPU.get().is_some() {
        return;
    }
    if let Some(ctx) = GpuNoiseContext::try_new_with_config(config) {
        let _ = GLOBAL_GPU.set(ctx);
    }
}

/// Returns `true` when a global GPU context has been initialised.
#[must_use]
pub fn has_global_gpu() -> bool {
    GLOBAL_GPU.get().is_some()
}

/// Returns a reference to the global GPU context, if initialised.
/// Used by other GPU-accelerated modules (surface, carver) that share the
/// same wgpu device.
#[must_use]
pub fn get_global_gpu() -> Option<&'static GpuNoiseContext> {
    GLOBAL_GPU.get()
}

/// GPU sky-light scan, using the global GPU context.
///
/// Returns `Some(Vec<u8>)` with one u8 per block position (flattened
/// `[column][y]`) when the GPU path succeeds, or `None` when no GPU is
/// available — the caller must fall back to the CPU scan.
///
/// # Panics
///
/// Panics if the global GPU context was never initialised. Call
/// [`has_global_gpu`] first if the caller can work without one.
#[must_use]
pub fn try_sky_light_gpu(input: &SkyLightInput) -> Option<Vec<u8>> {
    let ctx = GLOBAL_GPU
        .get()
        .expect("global GPU context not initialised");
    if input.total_positions() == 0 {
        return Some(Vec::new());
    }
    Some(ctx.scan_sky_light(input))
}

/// Uniform header for a light-scan dispatch.  Matches `LightDims` in
/// `light.wgsl`; the `padding` field keeps the struct 16-byte aligned so an
/// `array<LightDims>` in WGSL has natural alignment.
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct LightDims {
    pub num_columns: u32,
    pub height: u32,
    pub bottom_y: i32,
    pub padding: u32,
}

impl LightDims {
    #[must_use]
    pub const fn new(num_columns: u32, height: u32, bottom_y: i32) -> Self {
        Self {
            num_columns,
            height,
            bottom_y,
            padding: 0,
        }
    }
}

/// Input for a GPU sky-light scan over an 18×18×N block region.
///
/// The region is laid out as `[column * height + y]` where column iterates
/// `(x, z)` in row-major order over the 18×18 horizontal grid.
pub struct SkyLightInput {
    /// Per-block opacity values, flattened row-major: [column][y].
    pub opacity: Vec<u8>,
    /// Heightmap value per column (world Y of the top solid block).
    pub heightmap: Vec<i32>,
    /// Number of columns — typically 18 × 18 = 324.
    pub num_columns: u32,
    /// Number of Y levels (`max_y - bottom_y`).
    pub height: u32,
    /// World Y of the lowest block in the region.
    pub bottom_y: i32,
}

impl SkyLightInput {
    /// Total number of block positions in the scan region.
    #[must_use]
    pub const fn total_positions(&self) -> usize {
        self.num_columns as usize * self.height as usize
    }
}

/// Input for a GPU block-light scan.  Mirrors a pre-extracted luminance array.
pub struct BlockLightInput {
    /// Per-block luminance values, flattened row-major: [column][y].
    pub luminance: Vec<u8>,
    /// Number of columns — same as for sky light.
    pub num_columns: u32,
    /// Number of Y levels.
    pub height: u32,
    /// World Y of the lowest block in the region.
    pub bottom_y: i32,
}

impl BlockLightInput {
    #[must_use]
    pub const fn total_positions(&self) -> usize {
        self.num_columns as usize * self.height as usize
    }
}

/// A `SkyLightGpuFn`-compatible callback that uses the global GPU context.
///
/// Pass this to `pumpkin_world::lighting::register_sky_light_gpu` at server
/// startup to enable GPU-accelerated sky-light column scanning.
///
/// Returns `None` when no GPU is available (the caller falls back to CPU).
#[must_use]
pub fn sky_light_gpu_callback(
    opacity: &[u8],
    heightmap: &[i32],
    num_columns: u32,
    height: u32,
    bottom_y: i32,
) -> Option<Vec<u8>> {
    let ctx = GLOBAL_GPU.get()?;
    if num_columns == 0 || height == 0 {
        return Some(Vec::new());
    }
    // Skip the SkyLightInput allocation — pass slices directly to GPU.
    Some(ctx.scan_sky_light_raw(opacity, heightmap, num_columns, height, bottom_y))
}
