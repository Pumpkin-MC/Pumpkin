#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use criterion::{Criterion, criterion_group, criterion_main};
use pumpkin_world::{
    GlobalRandomConfig, bench_create_and_populate_biome, bench_create_and_populate_noise,
    bench_create_and_populate_noise_with_surface,
};

fn bench_terrain_gen(c: &mut Criterion) {
    let seed = 0;
    let random_config = GlobalRandomConfig::new(seed, false);

    c.bench_function("overworld biome", |b| {
        b.iter(|| {
            bench_create_and_populate_biome(&random_config);
        });
    });

    c.bench_function("overworld noise", |b| {
        b.iter(|| {
            bench_create_and_populate_noise(&random_config);
        });
    });

    c.bench_function("overworld surface", |b| {
        b.iter(|| {
            bench_create_and_populate_noise_with_surface(&random_config);
        });
    });
}

criterion_group!(benches, bench_terrain_gen);
criterion_main!(benches);
