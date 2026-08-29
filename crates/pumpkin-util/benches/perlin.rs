use criterion::{Criterion, criterion_group, criterion_main};
use pumpkin_util::noise::perlin::{OctavePerlinNoiseSampler, PerlinNoiseSampler};
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
    let sampler = PerlinNoiseSampler::new(&mut rand);
    let coords = make_coords(4096);

    c.bench_function("perlin_sample_no_fade", |b| {
        b.iter(|| {
            let mut acc = 0.0f64;
            for &(x, y, z) in &coords {
                acc += black_box(sampler.sample_no_fade(black_box(x), y, z, 0.0, 0.0));
            }
            black_box(acc)
        });
    });
}

fn bench_octave_perlin_sample(c: &mut Criterion) {
    let mut rand = Xoroshiro::from_seed(1);
    let (start, amplitudes) =
        OctavePerlinNoiseSampler::calculate_amplitudes(&(-4..=2).collect::<Vec<i32>>());
    let sampler = OctavePerlinNoiseSampler::new(&mut rand, start, &amplitudes, false);
    let coords = make_coords(4096);

    c.bench_function("octave_perlin_sample", |b| {
        b.iter(|| {
            let mut acc = 0.0f64;
            for &(x, y, z) in &coords {
                acc += black_box(sampler.sample(black_box(x), y, z));
            }
            black_box(acc)
        });
    });
}

criterion_group!(benches, bench_perlin_sample, bench_octave_perlin_sample);
criterion_main!(benches);
