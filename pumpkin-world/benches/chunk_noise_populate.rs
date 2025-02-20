use criterion::{Criterion, criterion_group, criterion_main};
use pumpkin_world::{
    GlobalProtoNoiseRouter, GlobalRandomConfig, NOISE_ROUTER_ASTS, bench_create_and_populate_noise,
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
