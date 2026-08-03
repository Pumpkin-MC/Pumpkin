//! World-generation logic compiled for GPU evaluation.
//!
//! The [`graph`] module lowers a density-function component stack into a flat
//! instruction buffer that the GPU compute shader (`graph.wgsl`) interprets.
//! The [`gpu`] module manages the wgpu device, pipeline and buffer lifecycle.
//!
//! It also provides a CPU-side reference interpreter used by tests to verify
//! GPU output within f32 tolerance.

pub mod gpu;
pub mod graph;
