#![cfg_attr(not(feature = "gpu-experimental"), allow(unused))]
#![allow(clippy::print_stdout)]

#[cfg(not(feature = "gpu-experimental"))]
fn main() {}

#[cfg(feature = "gpu-experimental")]
mod bench {
    use std::time::{Duration, Instant};

    use pumpkin_world::gpu_compute::{
        GpuNoiseEngine, NoiseAdapter, NoiseParams, density_field_cpu, density_field_cpu_parallel,
    };

    const HEIGHT: u32 = 384;
    const CHUNK_COUNTS: [u32; 5] = [1, 3, 5, 7, 9];
    const OCTAVES: [u32; 3] = [2, 6, 12];
    const REPS: usize = 5;

    fn ms(d: Duration) -> f64 {
        d.as_secs_f64() * 1000.0
    }

    fn mean(times: &[Duration]) -> f64 {
        times.iter().map(|d| ms(*d)).sum::<f64>() / times.len() as f64
    }

    fn min_ms(times: &[Duration]) -> f64 {
        times.iter().copied().min().map_or(0.0, ms)
    }

    const fn params(chunks: u32, octaves: u32) -> NoiseParams {
        NoiseParams {
            size_x: chunks * 16,
            size_y: HEIGHT,
            size_z: chunks * 16,
            octaves,
            seed: 0x5EED,
            frequency: 0.015,
            threshold: 0.0,
        }
    }

    struct Row {
        chunks: u32,
        voxels: usize,
        octaves: u32,
        backend: String,
        device: String,
        mean_ms: f64,
        min_ms: f64,
        extra: String,
    }

    impl Row {
        fn to_json(&self) -> String {
            format!(
                r#"{{"chunks":{},"voxels":{},"octaves":{},"backend":"{}","device":"{}","mean_ms":{:.4},"min_ms":{:.4}{}}}"#,
                self.chunks,
                self.voxels,
                self.octaves,
                self.backend,
                self.device,
                self.mean_ms,
                self.min_ms,
                self.extra
            )
        }
    }

    #[expect(clippy::too_many_lines)]
    pub fn main() {
        let mut engines = Vec::new();
        for (label, sel) in [
            ("igpu", NoiseAdapter::Integrated),
            ("dgpu", NoiseAdapter::Discrete),
        ] {
            match GpuNoiseEngine::new(sel) {
                Ok(e) => {
                    println!("{label}: using {}", e.device_name());
                    engines.push((label, e));
                }
                Err(e) => println!("{label}: UNAVAILABLE ({e})"),
            }
        }
        assert!(!engines.is_empty(), "no usable GPU adapters; aborting");

        let mut rows: Vec<Row> = Vec::new();

        for &octaves in &OCTAVES {
            for &chunks in &CHUNK_COUNTS {
                let p = params(chunks, octaves);
                let voxels = p.total();
                println!(
                    "\n=== {chunks}x{chunks} chunks | {voxels} voxels | {octaves} octaves ==="
                );

                let reference = density_field_cpu(&p);

                let mut serial = Vec::new();
                for _ in 0..REPS.min(if voxels > 3_000_000 { 2 } else { REPS }) {
                    let t = Instant::now();
                    let _ = density_field_cpu(&p);
                    serial.push(t.elapsed());
                }
                println!(
                    "  cpu-serial     mean {:.2} ms  min {:.2} ms",
                    mean(&serial),
                    min_ms(&serial)
                );
                rows.push(Row {
                    chunks,
                    voxels,
                    octaves,
                    backend: "cpu-serial".to_owned(),
                    device: "cpu".to_owned(),
                    mean_ms: mean(&serial),
                    min_ms: min_ms(&serial),
                    extra: String::new(),
                });

                let mut parallel = Vec::new();
                for _ in 0..REPS {
                    let t = Instant::now();
                    let out = density_field_cpu_parallel(&p);
                    parallel.push(t.elapsed());
                    assert_eq!(out, reference, "rayon path diverged from the serial path");
                }
                println!(
                    "  cpu-rayon      mean {:.2} ms  min {:.2} ms",
                    mean(&parallel),
                    min_ms(&parallel)
                );
                rows.push(Row {
                    chunks,
                    voxels,
                    octaves,
                    backend: "cpu-rayon".to_owned(),
                    device: "cpu".to_owned(),
                    mean_ms: mean(&parallel),
                    min_ms: min_ms(&parallel),
                    extra: String::new(),
                });

                for (label, engine) in &engines {
                    let mut out = vec![0u8; voxels];
                    let mut times = Vec::new();
                    let mut gpu = Duration::ZERO;
                    let mut back = Duration::ZERO;
                    let mut failed = None;
                    for _ in 0..REPS {
                        let t = Instant::now();
                        match engine.density_field(&p, &mut out) {
                            Ok(ph) => {
                                times.push(t.elapsed());
                                gpu += ph.gpu;
                                back += ph.readback;
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
                    let mismatches = reference.iter().zip(&out).filter(|(a, b)| a != b).count();
                    let frac = mismatches as f64 / voxels as f64;
                    assert!(
                        frac < 0.001,
                        "{label} classification differs on {:.4}% of voxels",
                        frac * 100.0
                    );
                    let n = times.len() as f64;
                    println!(
                        "  {label}           mean {:.2} ms  min {:.2} ms  [dispatch {:.2} | readback {:.2}]  ({:.4}% near-threshold disagreement)",
                        mean(&times),
                        min_ms(&times),
                        ms(gpu) / n,
                        ms(back) / n,
                        frac * 100.0
                    );
                    rows.push(Row {
                        chunks,
                        voxels,
                        octaves,
                        backend: (*label).to_owned(),
                        device: engine.device_name().to_owned(),
                        mean_ms: mean(&times),
                        min_ms: min_ms(&times),
                        extra: format!(
                            r#","dispatch_ms":{:.4},"readback_ms":{:.4},"mismatch_fraction":{frac:.6}"#,
                            ms(gpu) / n,
                            ms(back) / n
                        ),
                    });
                }
            }
        }

        let out = std::env::var("PUMPKIN_NOISE_BENCH_OUT")
            .unwrap_or_else(|_| "gpu_noise_results.json".to_owned());
        let json = format!(
            "[\n  {}\n]\n",
            rows.iter()
                .map(Row::to_json)
                .collect::<Vec<_>>()
                .join(",\n  ")
        );
        std::fs::write(&out, json).expect("failed to write results");
        println!("\nwrote {out}");
    }
}

#[cfg(feature = "gpu-experimental")]
fn main() {
    bench::main();
}
