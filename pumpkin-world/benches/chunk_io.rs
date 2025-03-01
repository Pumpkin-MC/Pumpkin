use std::{num::NonZeroU8, sync::Arc};

use criterion::{Criterion, criterion_group, criterion_main};
use pumpkin_util::math::vector2::Vector2;
use pumpkin_world::{cylindrical_chunk_iterator::Cylindrical, dimension::Dimension};
use temp_dir::TempDir;

fn criterion_benchmark(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let level = Arc::new(Dimension::OverWorld.into_level(temp_dir.path().to_path_buf()));

    c.bench_function("overworld chunk fetch", |b| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let cylindrical = Cylindrical::new(Vector2::new(0, 0), NonZeroU8::new(32).unwrap());
        let chunks = cylindrical.all_chunks_within();

        rt.block_on(async {
            let (tx, _rx) = tokio::sync::mpsc::channel(chunks.len());
            level.fetch_chunks(&chunks, tx).await;
            level.clean_chunks(&chunks).await;
        });

        b.to_async(rt).iter(|| {
            let (tx, _rx) = tokio::sync::mpsc::channel(chunks.len());

            level.fetch_chunks(&chunks, tx)
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
