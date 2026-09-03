use criterion::{Criterion, criterion_group, criterion_main};
use pumpkin_util::noise::perlin::{Noise, NormalNoise, PerlinNoise};
use pumpkin_util::noise::volume::DensityVolume;
use pumpkin_util::random::{RandomImpl, xoroshiro128::Xoroshiro};
use std::hint::black_box;

fn make_coords(count: usize) -> Vec<(f64, f64, f64)> {
    let mut rand = Xoroshiro::from_seed(0x5EED_C0DE_1234_5678);
    (0..count)
        .map(|_| {
            (
                (rand.next_f64() - 0.5) * 200_000.0,
                (rand.next_f64() - 0.5) * 4_000.0,
                (rand.next_f64() - 0.5) * 200_000.0,
            )
        })
        .collect()
}

fn bench_perlin_sample(c: &mut Criterion) {
    let mut rand = Xoroshiro::from_seed(1);
    let sampler = PerlinNoise::new(&mut rand);
    let coords = make_coords(4096);

    c.bench_function("perlin_sample", |b| {
        b.iter(|| {
            let mut acc = 0.0f32;
            for &(x, y, z) in &coords {
                acc += black_box(sampler.get(black_box(x), y, z));
            }
            black_box(acc)
        });
    });
}

fn bench_normal_noise_sample(c: &mut Criterion) {
    let mut rand = Xoroshiro::from_seed(1);
    let sampler = NormalNoise::new(&mut rand, -4, &[1.0; 7], false);
    let coords = make_coords(4096);

    c.bench_function("normal_noise_sample", |b| {
        b.iter(|| {
            let mut acc = 0.0f32;
            for &(x, y, z) in &coords {
                acc += black_box(sampler.get(black_box(x), y, z));
            }
            black_box(acc)
        });
    });
}

fn bench_normal_noise_volume(c: &mut Criterion) {
    let mut rand = Xoroshiro::from_seed(1);
    let sampler = NormalNoise::new(&mut rand, -4, &[1.0; 7], false);
    let volume = DensityVolume::with_block_step(16, 384, 16, 0, -64, 0);
    let mut buffer = vec![0.0f32; volume.size()];

    c.bench_function("normal_noise_volume", |b| {
        b.iter(|| {
            buffer.fill(0.0);
            sampler.add_to_volume(black_box(&mut buffer), &volume, 0.25, 0.25, 1.0);
            black_box(buffer[0])
        });
    });
}

criterion_group!(
    benches,
    bench_perlin_sample,
    bench_normal_noise_sample,
    bench_normal_noise_volume
);
criterion_main!(benches);
