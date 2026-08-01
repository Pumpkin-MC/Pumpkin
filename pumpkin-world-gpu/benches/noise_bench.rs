use criterion::{Criterion, criterion_group, criterion_main};
use pumpkin_util::{
    noise::perlin::OctavePerlinNoiseSampler,
    random::{RandomDeriverImpl, RandomGenerator, RandomImpl, xoroshiro128::Xoroshiro},
};
use pumpkin_world_gpu::{GpuNoiseContext, OctaveBatch};

fn make_reference_sampler() -> OctavePerlinNoiseSampler {
    let mut rand = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(1234));
    let splitter = rand.next_splitter();
    let mut rand = splitter.split_string("minecraft:terrain");
    let (first, amplitudes) =
        OctavePerlinNoiseSampler::calculate_amplitudes(&(-15..=0).collect::<Vec<i32>>());
    OctavePerlinNoiseSampler::new(&mut rand, first, &amplitudes, true)
}

fn make_points(count: usize) -> Vec<[f32; 3]> {
    (0..count)
        .map(|i| {
            let fx = f64::from(i as u32) * 3.7;
            let fy = f64::from(i as u32) * -1.9;
            let fz = f64::from(i as u32) * 5.3;
            [fx as f32, fy as f32, fz as f32]
        })
        .collect()
}

/// One 16x16 chunk's worth of noise-cell corners at vanilla's default horizontal (4)
/// and vertical (8) cell sizes, roughly what one `populate_noise` call sources from
/// `sample_start_density`/`sample_end_density`/`on_sampled_cell_corners` in
/// pumpkin-world/src/generation/proto_chunk.rs.
const CHUNK_CORNER_BATCH: usize = 5 * 5 * 48;

fn bench_noise(c: &mut Criterion) {
    let cpu_sampler = make_reference_sampler();
    let points = make_points(CHUNK_CORNER_BATCH);

    let mut group = c.benchmark_group("octave_noise_batch");
    group.bench_function("cpu_scalar", |b| {
        b.iter(|| {
            let mut total = 0.0f64;
            for p in &points {
                total += cpu_sampler.sample(f64::from(p[0]), f64::from(p[1]), f64::from(p[2]));
            }
            std::hint::black_box(total)
        });
    });

    // Naming the benchmark after the adapter puts it in criterion's own output, so
    // results stay interpretable without printing (the workspace denies print macros).
    if let Some(ctx) = GpuNoiseContext::try_new() {
        let batch = OctaveBatch::from_cpu_sampler(&cpu_sampler);
        let name = format!("gpu_wgpu [{}]", ctx.adapter_name);
        group.bench_function(name, |b| {
            b.iter(|| std::hint::black_box(ctx.sample_batch(&batch, &points)));
        });
    }

    group.finish();
}

criterion_group!(benches, bench_noise);
criterion_main!(benches);
