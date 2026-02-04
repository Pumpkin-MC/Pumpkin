use criterion::{Criterion, criterion_group, criterion_main};
use pumpkin_data::{
    chunk_gen_settings::GenerationSettings, dimension::Dimension,
    noise_router::OVERWORLD_BASE_NOISE_ROUTER,
};
use pumpkin_world::{
    GlobalRandomConfig, ProtoNoiseRouters, bench_create_and_populate_biome,
    bench_create_and_populate_noise, bench_create_and_populate_noise_with_surface,
    block::to_state_from_blueprint, generation::proto_chunk::TerrainCache,
};

fn bench_terrain_gen(c: &mut Criterion) {
    let seed = 0;
    let random_config = GlobalRandomConfig::new(seed, false);
    let base_router = ProtoNoiseRouters::generate(&OVERWORLD_BASE_NOISE_ROUTER, &random_config);
    let surface_config = GenerationSettings::from_dimension(&Dimension::OVERWORLD);
    let terrain_cache = TerrainCache::from_random(&random_config);
    let default_state = to_state_from_blueprint(&surface_config.default_block);

    c.bench_function("overworld biome", |b| {
        b.iter(|| {
            bench_create_and_populate_biome(
                &base_router,
                &random_config,
                surface_config,
                &terrain_cache,
                default_state,
            );
        });
    });

    c.bench_function("overworld noise", |b| {
        b.iter(|| {
            bench_create_and_populate_noise(
                &base_router,
                &random_config,
                surface_config,
                &terrain_cache,
                default_state,
            );
        });
    });

    c.bench_function("overworld surface", |b| {
        b.iter(|| {
            bench_create_and_populate_noise_with_surface(
                &base_router,
                &random_config,
                surface_config,
                &terrain_cache,
                default_state,
            );
        });
    });
}

criterion_group!(benches, bench_terrain_gen);
criterion_main!(benches);
