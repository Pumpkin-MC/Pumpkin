use criterion::{criterion_group, criterion_main, Criterion};
use pumpkin_world::{
    bench_create_and_populate_noise, GlobalProtoNoiseRouter, GlobalRandomConfig, NOISE_ROUTER_ASTS,
};

fn criterion_benchmark(c: &mut Criterion) {
    let seed = 0;
    let random_config = GlobalRandomConfig::new(seed);
    let base_router =
        GlobalProtoNoiseRouter::generate(NOISE_ROUTER_ASTS.overworld(), &random_config);

    c.bench_function("overworld noise", |b| {
        b.iter(|| bench_create_and_populate_noise(&base_router, &random_config));
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
