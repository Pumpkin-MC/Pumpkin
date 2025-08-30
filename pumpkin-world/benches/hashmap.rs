use criterion::{Criterion, criterion_group, criterion_main};
use pumpkin_util::math::vector2::Vector2;
use pumpkin_world::chunk_system::ChunkLoading;
use std::hint::black_box;
use std::sync::Arc;

fn bench_hashmap(c: &mut Criterion) {
    c.bench_function("ChunkLoading add 200*200 ticket", |b| {
        b.iter(|| {
            let mut map = ChunkLoading::new(Arc::new(Default::default()));
            for x in -100..100 {
                for y in -100..100 {
                    map.add_ticket(Vector2::new(x, y), 10);
                }
            }
            black_box(map);
        });
    });
    c.bench_function("ChunkLoading player 200 move", |b| {
        b.iter(|| {
            let mut map = ChunkLoading::new(Arc::new(Default::default()));
            for y in -100..100 {
                map.remove_ticket(Vector2::new(0, y - 1), 10);
                map.add_ticket(Vector2::new(0, y), 10);
            }
            black_box(map);
        });
    });
}

criterion_group!(benches, bench_hashmap);
criterion_main!(benches);
