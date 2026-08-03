//! GPU-side types and helpers for initial light scanning.
//!
//! The WGSL compute shader lives in `light.wgsl` alongside this file.
//! Dispatch is handled by [`super::gpu::GpuNoiseContext`].

use bytemuck::{Pod, Zeroable};

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
    pub fn total_positions(&self) -> usize {
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
    pub fn total_positions(&self) -> usize {
        self.num_columns as usize * self.height as usize
    }
}
