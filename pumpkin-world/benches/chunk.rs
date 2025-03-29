use criterion::{Criterion, criterion_group, criterion_main};
use pumpkin_world::{
    GENERATION_SETTINGS, GeneratorSetting, GlobalProtoNoiseRouter, GlobalRandomConfig,
    NOISE_ROUTER_ASTS, bench_create_and_populate_biome, bench_create_and_populate_noise,
    bench_create_and_populate_noise_with_surface,
};

fn bench_terrain_gen(c: &mut Criterion) {
    let seed = 0;
    let random_config = GlobalRandomConfig::new(seed, false);
    let base_router =
        GlobalProtoNoiseRouter::generate(NOISE_ROUTER_ASTS.overworld(), &random_config);
    let surface_config = GENERATION_SETTINGS
        .get(&GeneratorSetting::Overworld)
        .unwrap();

    c.bench_function("overworld biome", |b| {
        b.iter(|| bench_create_and_populate_biome(&base_router, &random_config, surface_config));
    });

    c.bench_function("overworld noise", |b| {
        b.iter(|| bench_create_and_populate_noise(&base_router, &random_config, surface_config));
    });

    c.bench_function("overworld surface", |b| {
        b.iter(|| {
            bench_create_and_populate_noise_with_surface(
                &base_router,
                &random_config,
                surface_config,
            )
        });
    });
}

criterion_group!(benches, bench_terrain_gen);
criterion_main!(benches);
