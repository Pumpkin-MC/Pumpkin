//! GPU-accelerated (wgpu) noise sampling prototype for `pumpkin-world`.
//!
//! Ports [`pumpkin_util::noise::perlin::OctavePerlinNoiseSampler`] to a WGSL compute
//! shader (see `octave_perlin.wgsl`) so a batch of independent sample points can be
//! evaluated in parallel instead of one at a time on the CPU.
//!
//! This intentionally runs in f32, not f64 like the CPU sampler: it will not reproduce
//! vanilla Minecraft terrain bit-for-bit for a given seed, only its own output
//! deterministically. GPU availability is never assumed — [`world::gpu::GpuNoiseContext::try_new`]
//! returns `None` when there is no compatible adapter, and callers must fall back to
//! the CPU sampler in that case.
//!
//! # What this can be worth end to end
//!
//! Measured with `cargo bench -p pumpkin-world --bench chunk_gen`, one overworld chunk
//! costs ~41.9 ms and splits as: lighting 37.4%, noise 35.9%, surface 9.4%, carvers
//! 5.2%, and the rest under 3% each. The stages account for 97% of the total, so the
//! split is trustworthy.
//!
//! This crate accelerates the noise stage, which caps the whole-chunk gain at **1.56x**
//! however fast the GPU gets — the measured 12-30x on the stage itself works out to
//! about 1.5x per chunk. Lighting is the larger share and is untouched; accelerating
//! both would raise the ceiling to ~3.7x. Chunk generation is also only part of what a
//! server does, so this does not translate into a server-wide multiplier.
//!
//! Worth knowing before deciding how much more to invest here.
//!
//! # Where f32 stops being a small difference
//!
//! For smooth density values f32 only costs a little precision. It is not harmless at
//! `RangeChoice` and `IntervalSelect` nodes: those compare a selector against a fixed
//! threshold, so one ulp of rounding does not shift the result slightly, it selects a
//! different subgraph.
//!
//! The overworld router hits this: seven of its threshold nodes read the same selector,
//! a Y gradient, against integer thresholds (-60, 51, 321) that real block coordinates
//! land on exactly. Vanilla computes that gradient as a lerp, which does not return an
//! integer Y exactly in f32 — and rounds differently per backend. It is lowered to
//! [`world::graph::OpCode::ClampedYIdentity`] instead, an exact clamp, which removes the
//! divergence. The `threshold_audit` example reports how exposed a router is.
//!
//! Any future node that feeds a threshold comparison needs the same scrutiny: agreeing
//! "to f32 precision" is not enough when the value is compared for ordering.
//!
//! Not every such case can be fixed. [`world::graph::OpCode::EndIslands`] decides whether an
//! island contributes with `simplex < -0.9`, and the simplex value is a genuine float
//! computation, so f32 and f64 legitimately land on opposite sides near the boundary
//! and an island appears or does not. Measured over a sweep of End coordinates that
//! affects roughly one point in 240. The Y-gradient case was fixable because the value
//! was exact in principle; this one is not.

pub mod world;

pub use world::gpu::{GpuNoiseContext, OctaveBatch, OctaveParams, PreparedGraph};
