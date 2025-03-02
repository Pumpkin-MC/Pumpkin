use std::{fs, num::NonZeroU8, path::PathBuf, sync::Arc};

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use pumpkin_util::math::{position, vector2::Vector2};
use pumpkin_world::{
    GlobalProtoNoiseRouter, GlobalRandomConfig, NOISE_ROUTER_ASTS, bench_create_and_populate_noise,
    chunk::ChunkData, cylindrical_chunk_iterator::Cylindrical, global_path, level::Level,
};
use temp_dir::TempDir;
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


async fn test_reads(level: Arc<Level>, positions: Vec<Vector2<i32>>) {
    let (send, mut recv) = tokio::sync::mpsc::channel(positions.len());

    let fetching_level = level.clone();
    tokio::spawn(async move { fetching_level.fetch_chunks(&positions, send).await });

    while let Some(x) = recv.recv().await {
        // Don't compile me away!
        let _ = x;
    }
    level.clean_memory();
}

async fn test_writes(level: Arc<Level>, chunks: Vec<(Vector2<i32>, Arc<RwLock<ChunkData>>)>) {
    level.write_chunks(chunks).await;
}

// Depends on config options from `./config`
fn bench_chunk_io(c: &mut Criterion) {
    // System temp dirs are in-memory, so we cant use temp_dir
    let temp_dir = TempDir::new().unwrap();
    let root_dir = temp_dir.path().to_path_buf();

    let async_handler = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let level = Arc::new(Level::from_root_folder(root_dir.clone()));

    println!("Initializing data...");
    // Initial writes
    let mut chunks = Vec::new();
    let mut positions = Vec::new();

    async_handler.block_on(async {
        let (send, mut recv) = tokio::sync::mpsc::channel(10);
        // Our data dir is empty, so we're generating new chunks here
        let level = level.clone();
        tokio::spawn(async move {
            let cylindrical = Cylindrical::new(Vector2::new(0, 0), NonZeroU8::new(32).unwrap());
            let chunk_positions = cylindrical.all_chunks_within();
            level.fetch_chunks(&chunk_positions, send).await;
            level.clean_chunks(&chunk_positions).await;
            level.clean_memory();
        });
        while let Some((chunk, _)) = recv.recv().await {
            let pos = chunk.read().await.position;
            chunks.push((pos, chunk));
            positions.push(pos);
        }
    });
    println!("Testing with {} chunks", chunks.len());

    chunks.sort_unstable_by_key(|chunk| chunk.0.x * chunk.0.x + chunk.0.z * chunk.0.z);
    positions.sort_unstable_by_key(|pos| pos.x * pos.x + pos.z * pos.z);

    // These test worst case: no caching done by `Level`
    for n_chunks in vec![8, 32, 128] {
        let chunks = &chunks[..n_chunks];
        let positions = &positions[..n_chunks];
        c.bench_with_input(
            BenchmarkId::new("write_chunks", n_chunks),
            &chunks,
            |b, chunks| {
                b.to_async(&async_handler)
                    .iter(|| test_writes(level.clone(), chunks.to_vec()))
            },
        );

        c.bench_with_input(
            BenchmarkId::new("read_chunks", n_chunks),
            &positions,
            |b, positions| {
                b.to_async(&async_handler)
                    .iter(|| test_reads(level.clone(), positions.to_vec()))
            },
        );
    }

    fs::remove_dir_all(&root_dir).unwrap();
}

criterion_group!(benches, bench_populate_noise, bench_chunk_io);
criterion_main!(benches);
