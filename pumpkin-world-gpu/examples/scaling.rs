//! Measures where GPU evaluation starts beating the CPU reference.
//!
//! Each dispatch pays a fixed setup cost (buffer creation, submission, readback), so
//! small batches lose and large ones win. One chunk's noise-cell corners is only ~1200
//! points, which lands on the losing side — the crossover is the number that decides
//! how many chunks a real integration has to batch per dispatch.
//!
//! Run with: cargo run -p pumpkin-world-gpu --release --example scaling

// Reporting the measurements to the console is the entire point of this example.
#![expect(clippy::print_stdout)]

use pumpkin_data::noise_router::NETHER_BASE_NOISE_ROUTER;
use pumpkin_world::generation::GlobalRandomConfig;
use pumpkin_world_gpu::GpuNoiseContext;
use pumpkin_world_gpu::graph::{BeardifierData, compile, evaluate_cpu};
use std::time::Instant;

fn main() {
    let config = GlobalRandomConfig::new(42, false);
    let stack = NETHER_BASE_NOISE_ROUTER.noise.full_component_stack;
    let compiled = compile(stack, &config).expect("nether compiles");
    let beard = BeardifierData::default();
    let ctx = GpuNoiseContext::try_new().expect("gpu");

    println!(
        "{:>10} {:>12} {:>12} {:>8}",
        "points", "cpu_ms", "gpu_ms", "speedup"
    );
    for &n in &[1_200usize, 5_000, 20_000, 80_000, 320_000] {
        let points: Vec<[f32; 3]> = (0..n)
            .map(|i| {
                let f = i as f32;
                [f * 0.7, (f % 97.0) * 1.3, f * 0.3]
            })
            .collect();

        // warm up
        let _ = ctx.evaluate_graph(&compiled, &points[..n.min(1000)]);

        let t = Instant::now();
        let mut total = 0.0f32;
        for p in &points {
            total += evaluate_cpu(
                &compiled.instructions,
                &compiled.samplers,
                &compiled.spline_points,
                &beard,
                p[0],
                p[1],
                p[2],
            );
        }
        std::hint::black_box(total);
        let cpu_ms = t.elapsed().as_secs_f64() * 1000.0;

        let t = Instant::now();
        std::hint::black_box(ctx.evaluate_graph(&compiled, &points));
        let gpu_ms = t.elapsed().as_secs_f64() * 1000.0;

        println!(
            "{n:>10} {cpu_ms:>12.2} {gpu_ms:>12.2} {:>8.2}x",
            cpu_ms / gpu_ms
        );
    }
}
