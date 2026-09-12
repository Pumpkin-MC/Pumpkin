#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use criterion::{Criterion, criterion_group, criterion_main};
use pumpkin_world::generation::structure::structures::create_chunk_random;
use pumpkin_world::generation::structure::structures::jigsaw::TemplatePool;
use std::hint::black_box;

const POOL: &str = "minecraft:ancient_city/structures";

fn bench_template_pool(c: &mut Criterion) {
    c.bench_function("template_pool/discover", |b| {
        b.iter(|| black_box(TemplatePool::discover(POOL)));
    });

    c.bench_function("template_pool/get_shuffled_elements", |b| {
        let pool = TemplatePool::discover(POOL).expect("pool exists");
        let mut random = create_chunk_random(42, 0, 0);
        b.iter(|| black_box(pool.get_shuffled_elements(&mut random)));
    });
}

criterion_group!(benches, bench_template_pool);
criterion_main!(benches);
