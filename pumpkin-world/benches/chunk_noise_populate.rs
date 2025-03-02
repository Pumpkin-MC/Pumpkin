use std::{fs, path::PathBuf, sync::Arc};

use criterion::{Criterion, criterion_group, criterion_main};
use pumpkin_util::math::vector2::Vector2;
use pumpkin_world::{
    GlobalProtoNoiseRouter, GlobalRandomConfig, NOISE_ROUTER_ASTS, bench_create_and_populate_noise,
    chunk::ChunkData, global_path, level::Level,
};
use tokio::sync::RwLock;

fn bench_populate_noise(c: &mut Criterion) {
    let seed = 0;
    let random_config = GlobalRandomConfig::new(seed);
    let base_router =
        GlobalProtoNoiseRouter::generate(NOISE_ROUTER_ASTS.overworld(), &random_config);

    c.bench_function("overworld noise", |b| {
        b.iter(|| bench_create_and_populate_noise(&base_router, &random_config));
    });
}

const MIN_POS: i32 = -4;
const MAX_POS: i32 = 4;

async fn test_reads(root_dir: PathBuf, positions: &[Vector2<i32>]) {
    let level = Level::from_root_folder(root_dir);

    let rt = tokio::runtime::Handle::current();
    let (send, mut recv) = tokio::sync::mpsc::channel(10);
    level.fetch_chunks(positions, send, &rt);
    while let Some(x) = recv.recv().await {
        // Don't compile me away!
        let _ = x;
    }
}

async fn test_writes(root_dir: PathBuf, chunks: &[(Vector2<i32>, Arc<RwLock<ChunkData>>)]) {
    let level = Level::from_root_folder(root_dir);
    for (pos, chunk) in chunks {
        level.write_chunk((*pos, chunk.clone())).await;
    }
}

// Depends on config options from `./config`
fn bench_chunk_io(c: &mut Criterion) {
    // System temp dirs are in-memory, so we cant use temp_dir
    let root_dir = global_path!("./bench_root");
    fs::create_dir(&root_dir).unwrap();

    let chunk_positions =
        (MIN_POS..=MAX_POS).flat_map(|x| (MIN_POS..=MAX_POS).map(move |z| Vector2::new(x, z)));
    let async_handler = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();

    println!("Initializing data...");
    // Initial writes
    let mut chunks = Vec::new();
    let mut positions = Vec::new();
    async_handler.block_on(async {
        let rt = tokio::runtime::Handle::current();
        let (send, mut recv) = tokio::sync::mpsc::channel(10);
        // Our data dir is empty, so we're generating new chunks here
        let level = Level::from_root_folder(root_dir.clone());
        level.fetch_chunks(&chunk_positions.collect::<Vec<_>>(), send, &rt);
        while let Some((chunk, _)) = recv.recv().await {
            let pos = chunk.read().await.position;
            chunks.push((pos, chunk));
            positions.push(pos);
        }
    });
    println!("Testing with {} chunks", chunks.len());

    // These test worst case: no caching done by `Level`
    c.bench_function("write chunks", |b| {
        b.to_async(&async_handler)
            .iter(|| test_writes(root_dir.clone(), &chunks))
    });

    c.bench_function("read chunks", |b| {
        b.to_async(&async_handler)
            .iter(|| test_reads(root_dir.clone(), &positions))
    });

    fs::remove_dir_all(&root_dir).unwrap();
}

criterion_group!(benches, bench_populate_noise, bench_chunk_io);
criterion_main!(benches);
