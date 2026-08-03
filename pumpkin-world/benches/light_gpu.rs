//! CPU vs iGPU vs dGPU benchmark for experimental GPU block-light propagation.
//! Empty without `--features gpu-experimental`, so the default build stays clean.
//!
//! `cargo bench -p pumpkin-world --features gpu-experimental --bench light_gpu`
//!
//! Results land as JSON in `$PUMPKIN_LIGHT_BENCH_OUT` (default
//! `light_gpu_bench.json`), which `plot_light_gpu.py` turns into a graph.

#![cfg_attr(not(feature = "gpu-experimental"), allow(unused))]
// A measurement harness, not server code. Printing numbers is the point.
#![allow(clippy::print_stdout)]

#[cfg(not(feature = "gpu-experimental"))]
fn main() {}

#[cfg(feature = "gpu-experimental")]
mod bench {
    use std::time::Instant;

    use pumpkin_world::lighting::gpu::{AdapterSelector, GpuLightEngine};
    use pumpkin_world::lighting::volume::{LightVolume, VoxelProps};

    const HEIGHT: u32 = 384;
    const CHUNK_COUNTS: [u32; 5] = [1, 3, 5, 7, 9];
    const REPS: usize = 5;

    /// Deterministic xorshift, so every backend sees identical input.
    struct Rng(u64);
    impl Rng {
        const fn next(&mut self) -> u64 {
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 7;
            self.0 ^= self.0 << 17;
            self.0
        }
        fn range(&mut self, n: u32) -> u32 {
            #[expect(clippy::cast_possible_truncation)]
            {
                (self.next() % u64::from(n)) as u32
            }
        }
    }

    /// An overworld-shaped region: stone under a rolling surface, carved cave
    /// bubbles, torches in those caves and on the surface. This is roughly what
    /// `LightEngine::initialize_light` faces on a freshly generated region.
    fn build_region(chunks: u32) -> (LightVolume, usize) {
        let side = chunks * 16;
        let total = (side as usize) * (HEIGHT as usize) * (side as usize);
        let mut props = vec![VoxelProps::default(); total];
        let idx = |x: u32, y: u32, z: u32| {
            ((y as usize) * (side as usize) + (z as usize)) * (side as usize) + (x as usize)
        };

        // Surface at world y in [60, 76] -> volume y in [124, 140] (bottom = -64).
        let mut heights = vec![0u32; (side * side) as usize];
        for z in 0..side {
            for x in 0..side {
                let fx = f64::from(x) * 0.08;
                let fz = f64::from(z) * 0.08;
                let h = 132.0 + 6.0 * fx.sin() + 6.0 * fz.cos() + 3.0 * (fx + fz).sin();
                #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                {
                    heights[(z * side + x) as usize] = h as u32;
                }
            }
        }

        for z in 0..side {
            for x in 0..side {
                let h = heights[(z * side + x) as usize];
                for y in 0..=h {
                    props[idx(x, y, z)].opacity = 15;
                }
            }
        }

        // Carve caves, about three bubbles per chunk.
        let mut rng = Rng(0x5EED_1234_ABCD_0001);
        let mut torches = 0usize;
        for _ in 0..(chunks * chunks * 3) {
            let cx = rng.range(side);
            let cz = rng.range(side);
            let cy = 70 + rng.range(50);
            let radius = 3 + rng.range(4);
            let r2 = (radius * radius) as i64;
            for dy in -(radius as i64)..=(radius as i64) {
                for dz in -(radius as i64)..=(radius as i64) {
                    for dx in -(radius as i64)..=(radius as i64) {
                        if dx * dx + dy * dy + dz * dz > r2 {
                            continue;
                        }
                        let (x, y, z) =
                            (i64::from(cx) + dx, i64::from(cy) + dy, i64::from(cz) + dz);
                        if x < 0
                            || z < 0
                            || y < 0
                            || x >= i64::from(side)
                            || z >= i64::from(side)
                            || y >= i64::from(HEIGHT)
                        {
                            continue;
                        }
                        #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                        {
                            props[idx(x as u32, y as u32, z as u32)].opacity = 0;
                        }
                    }
                }
            }
            // Torch at each bubble centre.
            props[idx(cx, cy, cz)].luminance = 14;
            torches += 1;
        }

        // Surface torches, one per chunk.
        for _ in 0..(chunks * chunks) {
            let x = rng.range(side);
            let z = rng.range(side);
            let y = heights[(z * side + x) as usize] + 1;
            props[idx(x, y, z)].luminance = 14;
            torches += 1;
        }

        (LightVolume::new(side, HEIGHT, side, &props), torches)
    }

    fn ms(d: std::time::Duration) -> f64 {
        d.as_secs_f64() * 1000.0
    }

    #[expect(clippy::too_many_lines)]
    pub fn main() {
        println!("Vulkan physical devices:");
        match pumpkin_world::lighting::gpu::list_devices() {
            Ok(devices) => {
                for (name, ty) in devices {
                    println!("  {ty:?}  {name}");
                }
            }
            Err(e) => println!("  enumeration failed: {e}"),
        }

        let mut engines = Vec::new();
        for (label, sel) in [
            ("igpu", AdapterSelector::Integrated),
            ("dgpu", AdapterSelector::Discrete),
        ] {
            match GpuLightEngine::new(sel) {
                Ok(e) => {
                    let d = e.device_info();
                    println!(
                        "{label}: using {} (readback memory {:?})",
                        d.name, d.readback_memory
                    );
                    engines.push((label, e));
                }
                Err(e) => println!("{label}: UNAVAILABLE ({e})"),
            }
        }
        assert!(!engines.is_empty(), "no usable GPU adapters; aborting");

        let mut rows = Vec::new();

        for &chunks in &CHUNK_COUNTS {
            let (mut volume, torches) = build_region(chunks);
            let voxels = volume.voxel_count();
            println!(
                "\n=== {chunks}x{chunks} chunks | {} x {HEIGHT} x {} = {voxels} voxels | {torches} emitters ===",
                volume.size_x, volume.size_z
            );

            // Gate on the smallest region: the order-free reference must agree.
            if chunks == 1 {
                volume.reset_light();
                volume.propagate_cpu();
                let cpu_ref = volume.light.clone();
                volume.reset_light();
                volume.propagate_reference();
                assert_eq!(
                    cpu_ref, volume.light,
                    "CPU BFS disagrees with relaxation reference"
                );
                println!("  correctness: CPU BFS == relaxation reference");
            }

            let mut cpu_times = Vec::new();
            for _ in 0..REPS {
                volume.reset_light();
                let t = Instant::now();
                volume.propagate_cpu();
                cpu_times.push(t.elapsed());
            }
            let cpu_light = volume.light.clone();
            let cpu_ms = summarize(&cpu_times);
            println!("  cpu   min {:.2} ms  mean {:.2} ms", cpu_ms.0, cpu_ms.1);
            rows.push(row(chunks, voxels, torches, "cpu", "cpu-bfs", cpu_ms));

            for (label, engine) in &engines {
                let mut times = Vec::new();
                let mut phases = Vec::new();
                let mut failed = None;
                for _ in 0..REPS {
                    volume.reset_light();
                    let t = Instant::now();
                    match engine.propagate(&mut volume) {
                        Ok(p) => {
                            times.push(t.elapsed());
                            phases.push(p);
                        }
                        Err(e) => {
                            failed = Some(e.to_string());
                            break;
                        }
                    }
                }
                if let Some(e) = failed {
                    println!("  {label}: FAILED ({e})");
                    continue;
                }
                let mismatches = cpu_light
                    .iter()
                    .zip(&volume.light)
                    .filter(|(a, b)| a != b)
                    .count();
                assert_eq!(
                    mismatches, 0,
                    "{label} produced {mismatches} light values differing from CPU"
                );
                let m = summarize(&times);
                let n = phases.len() as f64;
                let up = phases.iter().map(|p| ms(p.upload)).sum::<f64>() / n;
                let comp = phases.iter().map(|p| ms(p.compute)).sum::<f64>() / n;
                let back = phases.iter().map(|p| ms(p.readback)).sum::<f64>() / n;
                println!(
                    "  {label}   min {:.2} ms  mean {:.2} ms  (exact match vs CPU)  [upload {up:.2} | compute {comp:.2} | readback {back:.2}]",
                    m.0, m.1
                );
                rows.push(format!(
                    "{},\"upload_ms\":{up:.4},\"compute_ms\":{comp:.4},\"readback_ms\":{back:.4}}}",
                    row(
                        chunks,
                        voxels,
                        torches,
                        label,
                        &engine.device_info().name,
                        m
                    )
                    .trim_end_matches('}')
                ));
            }
        }

        let out = std::env::var("PUMPKIN_LIGHT_BENCH_OUT")
            .unwrap_or_else(|_| "light_gpu_bench.json".to_owned());
        let json = format!("[\n  {}\n]\n", rows.join(",\n  "));
        std::fs::write(&out, json).expect("failed to write results");
        println!("\nwrote {out}");
    }

    fn summarize(times: &[std::time::Duration]) -> (f64, f64) {
        let min = times.iter().copied().min().unwrap_or_default();
        let mean = times.iter().map(|d| ms(*d)).sum::<f64>() / times.len() as f64;
        (ms(min), mean)
    }

    fn row(
        chunks: u32,
        voxels: usize,
        emitters: usize,
        backend: &str,
        device: &str,
        m: (f64, f64),
    ) -> String {
        format!(
            r#"{{"chunks":{chunks},"voxels":{voxels},"emitters":{emitters},"backend":"{backend}","device":"{device}","min_ms":{:.4},"mean_ms":{:.4}}}"#,
            m.0, m.1
        )
    }
}

#[cfg(feature = "gpu-experimental")]
fn main() {
    bench::main();
}
