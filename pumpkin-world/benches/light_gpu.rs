#![cfg_attr(not(feature = "gpu-experimental"), allow(unused))]
#![allow(clippy::print_stdout)]

#[cfg(not(feature = "gpu-experimental"))]
fn main() {}

#[cfg(feature = "gpu-experimental")]
mod bench {
    use std::time::{Duration, Instant};

    use pumpkin_world::lighting::gpu::{AdapterSelector, GpuLightEngine};
    use pumpkin_world::lighting::volume::{LightVolume, PropDelta, VoxelProps};

    const HEIGHT: u32 = 384;
    const CHUNK_COUNTS: [u32; 5] = [1, 3, 5, 7, 9];
    const REPS: usize = 5;
    const TICKS: usize = 24;
    const ADDS_PER_TICK: usize = 2;
    const TORCH: u8 = 14;

    struct Rng(u64);
    impl Rng {
        const fn next(&mut self) -> u64 {
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 7;
            self.0 ^= self.0 << 17;
            self.0
        }
        fn range(&mut self, n: u32) -> u32 {
            (self.next() % u64::from(n)) as u32
        }
    }

    struct Region {
        props: Vec<VoxelProps>,
        side: u32,
        torches: usize,
        air_pool: Vec<u32>,
    }

    fn build_region(chunks: u32) -> Region {
        let side = chunks * 16;
        let total = (side as usize) * (HEIGHT as usize) * (side as usize);
        let mut props = vec![VoxelProps::default(); total];
        let idx = |x: u32, y: u32, z: u32| {
            ((y as usize) * (side as usize) + (z as usize)) * (side as usize) + (x as usize)
        };

        let mut heights = vec![0u32; (side * side) as usize];
        for z in 0..side {
            for x in 0..side {
                let fx = f64::from(x) * 0.08;
                let fz = f64::from(z) * 0.08;
                let h = 132.0 + 6.0 * fx.sin() + 6.0 * fz.cos() + 3.0 * (fx + fz).sin();
                heights[(z * side + x) as usize] = h as u32;
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

        let mut rng = Rng(0x5EED_1234_ABCD_0001);
        let mut torches = 0usize;
        let mut air_pool = Vec::new();
        let mut carved = 0usize;
        for _ in 0..(chunks * chunks * 3) {
            let cx = rng.range(side);
            let cz = rng.range(side);
            let cy = 70 + rng.range(50);
            let radius = 3 + rng.range(4);
            let r2 = i64::from(radius * radius);
            for dy in -i64::from(radius)..=i64::from(radius) {
                for dz in -i64::from(radius)..=i64::from(radius) {
                    for dx in -i64::from(radius)..=i64::from(radius) {
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
                        let i = idx(x as u32, y as u32, z as u32);
                        props[i].opacity = 0;
                        carved += 1;
                        if carved.is_multiple_of(53) {
                            air_pool.push(i as u32);
                        }
                    }
                }
            }
            props[idx(cx, cy, cz)].luminance = TORCH;
            torches += 1;
        }

        for _ in 0..(chunks * chunks) {
            let x = rng.range(side);
            let z = rng.range(side);
            let y = heights[(z * side + x) as usize] + 1;
            props[idx(x, y, z)].luminance = TORCH;
            torches += 1;
        }

        Region {
            props,
            side,
            torches,
            air_pool,
        }
    }

    fn volume_of(region: &Region) -> LightVolume {
        LightVolume::new(region.side, HEIGHT, region.side, &region.props)
    }

    fn build_ticks(region: &Region) -> Vec<Vec<PropDelta>> {
        let lit = VoxelProps {
            opacity: 0,
            luminance: TORCH,
        };
        let dark = VoxelProps {
            opacity: 0,
            luminance: 0,
        };
        let mut rng = Rng(0x0BAD_F00D_1234_5678);
        let mut placed: Vec<u32> = Vec::new();
        let mut ticks = Vec::with_capacity(TICKS);
        if region.air_pool.is_empty() {
            return ticks;
        }
        for _ in 0..TICKS {
            let mut tick: Vec<PropDelta> = Vec::new();
            for _ in 0..ADDS_PER_TICK {
                let pick = region.air_pool[rng.range(region.air_pool.len() as u32) as usize];
                if !placed.contains(&pick) && !tick.iter().any(|d| d.0 == pick) {
                    placed.push(pick);
                    tick.push((pick, lit));
                }
            }
            while placed.len() > ADDS_PER_TICK * 2 {
                let old = placed.remove(0);
                if !tick.iter().any(|d| d.0 == old) {
                    tick.push((old, dark));
                }
            }
            if !tick.is_empty() {
                ticks.push(tick);
            }
        }
        ticks
    }

    fn ms(d: Duration) -> f64 {
        d.as_secs_f64() * 1000.0
    }

    fn mean(times: &[Duration]) -> f64 {
        if times.is_empty() {
            return 0.0;
        }
        times.iter().map(|d| ms(*d)).sum::<f64>() / times.len() as f64
    }

    fn min_ms(times: &[Duration]) -> f64 {
        times.iter().copied().min().map_or(0.0, ms)
    }

    struct Row {
        chunks: u32,
        voxels: usize,
        emitters: usize,
        backend: String,
        device: String,
        mode: &'static str,
        min_ms: f64,
        mean_ms: f64,
        extra: String,
    }

    impl Row {
        fn to_json(&self) -> String {
            format!(
                r#"{{"chunks":{},"voxels":{},"emitters":{},"backend":"{}","device":"{}","mode":"{}","min_ms":{:.4},"mean_ms":{:.4}{}}}"#,
                self.chunks,
                self.voxels,
                self.emitters,
                self.backend,
                self.device,
                self.mode,
                self.min_ms,
                self.mean_ms,
                self.extra
            )
        }
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

        let mut rows: Vec<Row> = Vec::new();

        for &chunks in &CHUNK_COUNTS {
            let region = build_region(chunks);
            let ticks = build_ticks(&region);
            let mut volume = volume_of(&region);
            let voxels = volume.voxel_count();
            println!(
                "\n=== {chunks}x{chunks} chunks | {} x {HEIGHT} x {} = {voxels} voxels | {} emitters | {} ticks ===",
                region.side,
                region.side,
                region.torches,
                ticks.len()
            );

            volume.propagate_cpu();
            let baseline = volume.light.clone();
            let mut check = volume_of(&region);
            check.propagate_reference();
            assert_eq!(
                baseline, check.light,
                "CPU BFS disagrees with the relaxation reference on this scene"
            );
            drop(check);
            println!("  correctness: CPU BFS == relaxation reference on the base scene");

            let mut full_times = Vec::new();
            for _ in 0..REPS {
                let mut v = volume_of(&region);
                let t = Instant::now();
                v.propagate_cpu();
                full_times.push(t.elapsed());
            }
            println!(
                "  cpu-full       min {:.2} ms  mean {:.2} ms   (recompute the whole region)",
                min_ms(&full_times),
                mean(&full_times)
            );
            rows.push(Row {
                chunks,
                voxels,
                emitters: region.torches,
                backend: "cpu-full".to_owned(),
                device: "cpu".to_owned(),
                mode: "full",
                min_ms: min_ms(&full_times),
                mean_ms: mean(&full_times),
                extra: String::new(),
            });

            let mut cpu_delta_volume = volume_of(&region);
            cpu_delta_volume.propagate_cpu();
            let mut cpu_delta_times = Vec::new();
            for tick in &ticks {
                let t = Instant::now();
                cpu_delta_volume.propagate_delta(tick);
                cpu_delta_times.push(t.elapsed());
            }
            let cpu_delta_final = cpu_delta_volume.light.clone();
            println!(
                "  cpu-delta      min {:.2} ms  mean {:.2} ms   (per tick)",
                min_ms(&cpu_delta_times),
                mean(&cpu_delta_times)
            );
            rows.push(Row {
                chunks,
                voxels,
                emitters: region.torches,
                backend: "cpu-delta".to_owned(),
                device: "cpu".to_owned(),
                mode: "delta",
                min_ms: min_ms(&cpu_delta_times),
                mean_ms: mean(&cpu_delta_times),
                extra: String::new(),
            });

            for (label, engine) in &engines {
                let mut gv = volume_of(&region);
                let mut full_gpu_times = Vec::new();
                let mut failed = None;
                for tick in &ticks {
                    gv.apply_prop_deltas(tick);
                    gv.reset_light();
                    let t = Instant::now();
                    match engine.propagate(&mut gv) {
                        Ok(_) => full_gpu_times.push(t.elapsed()),
                        Err(e) => {
                            failed = Some(e.to_string());
                            break;
                        }
                    }
                }
                if let Some(e) = failed {
                    println!("  {label}-full: FAILED ({e})");
                } else {
                    let mismatches = cpu_delta_final
                        .iter()
                        .zip(&gv.light)
                        .filter(|(a, b)| a != b)
                        .count();
                    assert_eq!(
                        mismatches, 0,
                        "{label} full-reupload path differs from the CPU delta solve"
                    );
                    println!(
                        "  {label}-full      min {:.2} ms  mean {:.2} ms   (per tick, whole grid up and down)",
                        min_ms(&full_gpu_times),
                        mean(&full_gpu_times)
                    );
                    rows.push(Row {
                        chunks,
                        voxels,
                        emitters: region.torches,
                        backend: format!("{label}-full"),
                        device: engine.device_info().name.clone(),
                        mode: "full",
                        min_ms: min_ms(&full_gpu_times),
                        mean_ms: mean(&full_gpu_times),
                        extra: String::new(),
                    });
                }

                let mut rv = volume_of(&region);
                let init = Instant::now();
                let resident = match engine.resident(&mut rv, 64) {
                    Ok(r) => r,
                    Err(e) => {
                        println!("  {label}-resident: FAILED ({e})");
                        continue;
                    }
                };
                let init_ms = ms(init.elapsed());
                let resident_mb = resident.resident_bytes() as f64 / (1024.0 * 1024.0);

                let mut res_times = Vec::new();
                let mut host = Duration::ZERO;
                let mut gpu = Duration::ZERO;
                let mut back = Duration::ZERO;
                let mut dispatch_frac = 0.0f64;
                let mut read_bytes = 0usize;
                let mut failed = None;
                for tick in &ticks {
                    let t = Instant::now();
                    match resident.apply_deltas(&mut rv, tick) {
                        Ok(p) => {
                            res_times.push(t.elapsed());
                            host += p.host;
                            gpu += p.gpu;
                            back += p.readback;
                            dispatch_frac += p.dispatch_voxels as f64 / voxels as f64;
                            read_bytes += p.readback_bytes;
                        }
                        Err(e) => {
                            failed = Some(e.to_string());
                            break;
                        }
                    }
                }
                if let Some(e) = failed {
                    println!("  {label}-resident: FAILED ({e})");
                    continue;
                }
                let mismatches = cpu_delta_final
                    .iter()
                    .zip(&rv.light)
                    .filter(|(a, b)| a != b)
                    .count();
                assert_eq!(
                    mismatches, 0,
                    "{label} resident path differs from the CPU delta solve"
                );
                let n = res_times.len() as f64;
                println!(
                    "  {label}-resident min {:.2} ms  mean {:.2} ms   (per tick) [host {:.3} | gpu {:.3} | read {:.3}]  dispatch {:.1}% of grid, {resident_mb:.1} MB resident, {init_ms:.1} ms init",
                    min_ms(&res_times),
                    mean(&res_times),
                    ms(host) / n,
                    ms(gpu) / n,
                    ms(back) / n,
                    dispatch_frac / n * 100.0,
                );
                rows.push(Row {
                    chunks,
                    voxels,
                    emitters: region.torches,
                    backend: format!("{label}-resident"),
                    device: engine.device_info().name.clone(),
                    mode: "delta",
                    min_ms: min_ms(&res_times),
                    mean_ms: mean(&res_times),
                    extra: format!(
                        r#","host_ms":{:.4},"gpu_ms":{:.4},"readback_ms":{:.4},"dispatch_fraction":{:.5},"readback_bytes":{},"resident_mb":{resident_mb:.2},"init_ms":{init_ms:.3}"#,
                        ms(host) / n,
                        ms(gpu) / n,
                        ms(back) / n,
                        dispatch_frac / n,
                        read_bytes / res_times.len(),
                    ),
                });
            }
        }

        let out = std::env::var("PUMPKIN_LIGHT_BENCH_OUT")
            .unwrap_or_else(|_| "light_gpu_delta_results.json".to_owned());
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
