//! Full-chunk noise evaluation on the GPU.
//!
//! Wraps [`compile_router`](super::graph::compile_router) + [`PreparedGraph`]
//! into a reusable pipeline that evaluates every density-function output at
//! arbitrary 3D positions.  Callers feed in block/corner coordinates and
//! receive the 10 router outputs per point in one dispatch — just like the
//! [`scaling`](https://github.com/Pumpkin-MC/Pumpkin/blob/GPU-accelerated/pumpkin-gpu/examples/scaling.rs)
//! example, but packaged for production use.
//!
//! # Quick start
//!
//! ```ignore
//! use pumpkin_gpu::world::chunk::OverworldNoisePipeline;
//! use pumpkin_gpu::GpuNoiseContext;
//!
//! let ctx = GpuNoiseContext::try_new().unwrap();
//! let pipeline = OverworldNoisePipeline::new(42, false).unwrap();
//!
//! // Generate all block positions for chunk (0, 0):
//! let positions = OverworldNoisePipeline::chunk_block_positions(0, 0, -64, 320);
//! let outputs = pipeline.evaluate(&ctx, &positions);
//! // outputs.len() == positions.len() * 10
//! ```

use super::gpu::GpuNoiseContext;
use super::graph::{BeardifierData, CompiledGraph, compile_router, output_slot::FINAL_DENSITY};
use pumpkin_data::noise_router::OVERWORLD_BASE_NOISE_ROUTER;
use pumpkin_world::generation::GlobalRandomConfig;

/// Pre-compiled overworld density-function graph, ready for repeated GPU
/// evaluation at arbitrary point batches.
#[derive(Debug)]
pub struct OverworldNoisePipeline {
    compiled: CompiledGraph,
}

impl OverworldNoisePipeline {
    /// Compiles the vanilla overworld router for the given seed+legacy flag.
    ///
    /// Returns `Err(unsupported_node)` when the router contains a node that
    /// the GPU compiler cannot lower (currently only `FindTopSurface`).
    pub fn new(seed: u64, legacy: bool) -> Result<Self, super::graph::UnsupportedNode> {
        let config = GlobalRandomConfig::new(seed, legacy);
        let compiled = compile_router(&OVERWORLD_BASE_NOISE_ROUTER.noise, &config)?;
        Ok(Self { compiled })
    }

    /// Number of f32 values the GPU returns per input point (always 10 for
    /// the overworld router).
    #[must_use]
    pub fn outputs_per_point(&self) -> usize {
        self.compiled.outputs_per_point()
    }

    /// Evaluate the density-function graph on the GPU for all `positions`,
    /// returning `outputs_per_point()` values per point in row-major order.
    ///
    /// The 10 outputs per point are laid out as:
    ///
    /// | Index | Name |
    /// |-------|------|
    /// | 0 | `barrier_noise` |
    /// | 1 | `fluid_level_floodedness_noise` |
    /// | 2 | `fluid_level_spread_noise` |
    /// | 3 | `lava_noise` |
    /// | 4 | `erosion` |
    /// | 5 | `depth` |
    /// | 6 | `final_density` |
    /// | 7 | `vein_toggle` |
    /// | 8 | `vein_ridged` |
    /// | 9 | `vein_gap` |
    #[must_use]
    pub fn evaluate(&self, ctx: &GpuNoiseContext, positions: &[[f32; 3]]) -> Vec<f32> {
        ctx.evaluate_graph_with(&self.compiled, positions, &BeardifierData::default())
    }

    /// Like [`evaluate`](Self::evaluate) but uses a [`PreparedGraph`] for
    /// repeated dispatches (uploads graph tables once).
    #[must_use]
    pub fn prepare<'a>(&self, ctx: &'a GpuNoiseContext) -> super::gpu::PreparedGraph<'a> {
        ctx.prepare(&self.compiled)
    }

    /// Generate all block positions for one chunk.
    ///
    /// Yields `(x, y, z)` as `[f32; 3]` for every block in the 16×16×(max_y-min_y)
    /// volume in Z-outer, Y-middle, X-inner order (matching the CPU populate
    /// loop so indices align naturally).
    #[must_use]
    pub fn chunk_block_positions(
        chunk_x: i32,
        chunk_z: i32,
        min_y: i32,
        max_y: i32,
    ) -> Vec<[f32; 3]> {
        let base_x = chunk_x * 16;
        let base_z = chunk_z * 16;
        let count = 16 * 16 * (max_y - min_y) as usize;
        let mut positions = Vec::with_capacity(count);

        for z in 0..16 {
            let wz = (base_z + z) as f32;
            for y in min_y..max_y {
                let wy = y as f32;
                for x in 0..16 {
                    positions.push([(base_x + x) as f32, wy, wz]);
                }
            }
        }
        positions
    }

    /// Collect the values for one output slot from an output-major result buffer
    /// (layout: `[slot][point]`, i.e. all points for slot 0, then all for slot 1, …).
    #[must_use]
    pub fn collect_output(results: &[f32], slot: usize, num_points: usize) -> Vec<f32> {
        let start = slot * num_points;
        results[start..start + num_points].to_vec()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn chunk_block_positions_count() {
        let positions = OverworldNoisePipeline::chunk_block_positions(0, 0, -64, 320);
        assert_eq!(positions.len(), 16 * 16 * 384); // 98,304
        // First position: (0, -64, 0)
        assert_eq!(positions[0], [0.0, -64.0, 0.0]);
        // Last position: (15, 319, 15)
        assert_eq!(positions.last().unwrap(), &[15.0, 319.0, 15.0]);
    }

    #[test]
    fn overworld_router_compiles() {
        let pipeline = OverworldNoisePipeline::new(42, false);
        assert!(
            pipeline.is_ok(),
            "overworld router did not compile: {pipeline:?}"
        );
        let p = pipeline.unwrap();
        assert_eq!(p.outputs_per_point(), 10);
    }

    #[test]
    fn gpu_evaluates_all_chunk_positions() {
        let Some(ctx) = GpuNoiseContext::try_new() else {
            return;
        };
        let pipeline = OverworldNoisePipeline::new(42, false).expect("overworld lowers");
        let positions = OverworldNoisePipeline::chunk_block_positions(0, 0, 0, 16); // small slice

        let results = pipeline.evaluate(&ctx, &positions);
        let expected_len = positions.len() * 10;
        assert_eq!(results.len(), expected_len);

        // final_density (slot 6) should not be all zero
        let densities =
            OverworldNoisePipeline::collect_output(&results, FINAL_DENSITY, positions.len());
        let nonzero = densities.iter().filter(|&&d| d != 0.0).count();
        assert!(
            nonzero > positions.len() / 2,
            "only {nonzero}/{len} final_density values non-zero; expected most to be non-zero",
            len = densities.len()
        );
    }

    /// End-to-end: GPU chunk pipeline vs CPU reference on real block positions.
    /// Tests that `OverworldNoisePipeline` produces results consistent with the
    /// CPU reference evaluator across all 10 router outputs.
    #[test]
    fn gpu_chunk_pipeline_matches_cpu_reference() {
        let Some(ctx) = GpuNoiseContext::try_new() else {
            return;
        };
        let pipeline = OverworldNoisePipeline::new(42, false).expect("overworld lowers");
        // 4×4×16 = 256 positions — enough to cover all cells in a small slice
        let positions = OverworldNoisePipeline::chunk_block_positions(0, 0, 0, 16);

        let gpu_results = pipeline.evaluate(&ctx, &positions);
        let stride = pipeline.outputs_per_point();
        assert_eq!(gpu_results.len(), positions.len() * stride);

        let beard = BeardifierData::default();
        let mut max_diff = [0.0f32; 10];
        let mut nonzero = [0usize; 10];

        // GPU result layout is output-major: [slot][point], not [point][slot].
        for (pi, point) in positions.iter().enumerate() {
            for slot in 0..stride {
                let node = pipeline.compiled.outputs[slot];
                let gpu_val = gpu_results[slot * positions.len() + pi];
                let cpu_val = crate::world::graph::evaluate_cpu_node(
                    &pipeline.compiled,
                    &beard,
                    node,
                    point[0],
                    point[1],
                    point[2],
                );
                let diff = (gpu_val - cpu_val).abs();
                max_diff[slot] = max_diff[slot].max(diff);
                if cpu_val != 0.0 {
                    nonzero[slot] += 1;
                }
            }
        }

        let slot_names = [
            "barrier_noise",
            "fluid_level_floodedness_noise",
            "fluid_level_spread_noise",
            "lava_noise",
            "erosion",
            "depth",
            "final_density",
            "vein_toggle",
            "vein_ridged",
            "vein_gap",
        ];
        // Tolerances per output, matching the f32-vs-f64 gap measured by the
        // per-opcode tests in gpu.rs. Barrier noise involves Spline→Noise chains
        // (worst gap), final density accumulates many operations (moderate gap),
        // simpler outputs are tighter.
        let tolerances: [f32; 10] = [
            5e-2, // barrier_noise: spline → noise, largest f32 drift
            1e-2, // fluid_level_floodedness_noise
            1e-2, // fluid_level_spread_noise
            1e-2, // lava_noise
            1e-2, // erosion
            1e-2, // depth
            1e-2, // final_density: accumulates many ops
            1e-4, // vein_toggle: simple threshold
            1e-2, // vein_ridged
            1e-4, // vein_gap: simple threshold
        ];
        for slot in 0..stride {
            assert!(
                max_diff[slot] < tolerances[slot],
                "{}: max GPU-CPU diff {:.6} exceeds tolerance {:.6}",
                slot_names[slot],
                max_diff[slot],
                tolerances[slot]
            );
        }
        // All outputs should have meaningful (non-zero) values.
        for slot in 0..stride {
            assert!(
                nonzero[slot] > 0,
                "{}: all values are zero",
                slot_names[slot]
            );
        }
    }

    #[test]
    fn extract_output_single_value() {
        // Output-major layout: [s0p0, s0p1, s0p2, s1p0, s1p1, s1p2, s2p0, s2p1, s2p2]
        let data = vec![1.0, 11.0, 21.0, 2.0, 12.0, 22.0, 3.0, 13.0, 23.0];
        let b = OverworldNoisePipeline::collect_output(&data, 1, 3);
        assert_eq!(b, vec![2.0, 12.0, 22.0]);
        let c = OverworldNoisePipeline::collect_output(&data, 2, 3);
        assert_eq!(c, vec![3.0, 13.0, 23.0]);
    }
}
