//! Measures where GPU evaluation starts beating the CPU reference.
//!
//! Each dispatch pays a fixed setup cost (buffer creation, submission, readback), so
//! small batches lose and large ones win. One chunk's noise-cell corners is only ~1200
//! points, which lands on the losing side — the crossover is the number that decides
//! how many chunks a real integration has to batch per dispatch.
//!
//! An integrated GPU ramps its clocks under sustained load, so the first run of this
//! example reports worse GPU times than the second or third. Run it a few times and
//! read the last one: that is the steady state a running server would see.
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
        "{:>10} {:>8} {:>10} {:>10} {:>10}",
        "points", "chunks", "cpu_ms", "gpu_ms", "speedup"
    );
    for &n in &[
        1_200usize, 2_400, 4_800, 9_600, 19_200, 38_400, 76_800, 320_000,
    ] {
        let points: Vec<[f32; 3]> = (0..n)
            .map(|i| {
                let f = i as f32;
                [f * 0.7, (f % 97.0) * 1.3, f * 0.3]
            })
            .collect();

        // Prepared once, as a real integration would.
        let mut prepared = ctx.prepare(&compiled);

        // Single timings are dominated by scheduling noise, so take the best of
        // several runs on both sides: that is the throughput each path can sustain.
        let iterations = if n > 100_000 { 3 } else { 10 };

        let mut cpu_ms = f64::INFINITY;
        for _ in 0..iterations {
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
            cpu_ms = cpu_ms.min(t.elapsed().as_secs_f64() * 1000.0);
        }

        let mut gpu_ms = f64::INFINITY;
        for _ in 0..iterations {
            let t = Instant::now();
            std::hint::black_box(prepared.evaluate(&points, &beard));
            gpu_ms = gpu_ms.min(t.elapsed().as_secs_f64() * 1000.0);
        }

        let chunks = n as f64 / 1200.0;
        println!(
            "{n:>10} {chunks:>8.1} {cpu_ms:>10.2} {gpu_ms:>10.2} {:>9.2}x",
            cpu_ms / gpu_ms
        );
    }
}
