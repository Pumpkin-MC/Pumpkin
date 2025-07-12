use criterion::{Criterion, criterion_group, criterion_main};
use pumpkin_data::noise_router::OVERWORLD_BASE_NOISE_ROUTER;
use pumpkin_world::generation::{
    GlobalRandomConfig,
    noise_router::{
        chunk_density_function::ChunkNoiseFunctionBuilderOptions,
        chunk_noise_router::ChunkNoiseRouter, proto_noise_router::ProtoNoiseRouters,
    },
};
use std::hint::black_box;

fn bench_noise_router_creation(c: &mut Criterion) {
    let base_routers = &OVERWORLD_BASE_NOISE_ROUTER;
    let random_config = GlobalRandomConfig::new(0, false);

    let proto_routers = ProtoNoiseRouters::generate(base_routers, &random_config);
    let proto_noise_router = proto_routers.noise;

    let builder_options = ChunkNoiseFunctionBuilderOptions::new(4, 8, 4, 4, 0, 0, 3);

    // Benchmarking
    c.bench_function("noise_router_creation_with_pooling", |b| {
        b.iter(|| {
            let router = ChunkNoiseRouter::generate(
                black_box(&proto_noise_router),
                black_box(&builder_options),
            );
            black_box(router);
        })
    });
}

criterion_group!(benches, bench_noise_router_creation);
criterion_main!(benches);
