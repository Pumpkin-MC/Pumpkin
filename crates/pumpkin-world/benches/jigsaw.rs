#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use criterion::{Criterion, criterion_group, criterion_main};
use pumpkin_data::Rotation;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::random::{RandomGenerator, xoroshiro128::Xoroshiro};
use pumpkin_world::generation::structure::structures::jigsaw::TemplatePool;
use std::hint::black_box;

/// Pools that drive large, deep jigsaw structures, so the per-element template
/// work (jigsaw lookup, bounding box, height) is exercised as in production.
const POOLS: [&str; 2] = [
    "minecraft:ancient_city/city_center",
    "minecraft:pillager_outpost/base_plates",
];

fn bench_jigsaw_pool_elements(c: &mut Criterion) {
    let offset = BlockPos::new(0, 0, 0);

    for id in POOLS {
        let pool = TemplatePool::discover(id).expect("pool should load");
        let kind = &pool.elements[0].kind;

        c.bench_function(&format!("jigsaw/get_shuffled_blocks/{id}"), |b| {
            b.iter(|| {
                let mut random = RandomGenerator::Xoroshiro(Xoroshiro::from_seed(black_box(0)));
                black_box(kind.get_shuffled_jigsaw_blocks(
                    black_box(offset),
                    Rotation::None,
                    &mut random,
                ));
            });
        });

        c.bench_function(&format!("jigsaw/get_y_size/{id}"), |b| {
            b.iter(|| black_box(kind.get_y_size()));
        });

        c.bench_function(&format!("jigsaw/get_bounding_box/{id}"), |b| {
            b.iter(|| black_box(kind.get_bounding_box(black_box(offset), Rotation::None)));
        });
    }
}

criterion_group!(benches, bench_jigsaw_pool_elements);
criterion_main!(benches);
