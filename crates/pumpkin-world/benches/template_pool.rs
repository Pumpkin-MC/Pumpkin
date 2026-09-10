#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use pumpkin_world::generation::structure::structures::create_chunk_random;
use pumpkin_world::generation::structure::structures::jigsaw::TemplatePool;

const POOL: &str = "minecraft:ancient_city/structures";

fn bench_template_pool(c: &mut Criterion) {
    let mut group = c.benchmark_group("template_pool");

    group.bench_function("discover", |b| {
        b.iter(|| black_box(TemplatePool::discover(POOL)));
    });

    group.bench_function("get_shuffled_elements", |b| {
        let pool = TemplatePool::discover(POOL).expect("pool exists");
        let mut random = create_chunk_random(42, 0, 0);
        b.iter(|| black_box(pool.get_shuffled_elements(&mut random)));
    });

    group.finish();
}

criterion_group!(benches, bench_template_pool);
criterion_main!(benches);
