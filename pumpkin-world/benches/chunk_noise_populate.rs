use std::{fs, num::NonZeroU8, path::PathBuf, sync::Arc};

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use pumpkin_util::math::vector2::Vector2;
use pumpkin_world::{
    GlobalProtoNoiseRouter, GlobalRandomConfig, NOISE_ROUTER_ASTS, bench_create_and_populate_noise,
    chunk::ChunkData, cylindrical_chunk_iterator::Cylindrical, global_path, level::Level,
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

async fn test_reads(root_dir: PathBuf, positions: Vec<Vector2<i32>>) {
    let level = Arc::new(Level::from_root_folder(root_dir));

    let (send, mut recv) = tokio::sync::mpsc::channel(positions.len());
    tokio::spawn(async move { level.fetch_chunks(&positions, send).await });

    while let Some(x) = recv.recv().await {
        // Don't compile me away!
        let _ = x;
    }
}

async fn test_writes(root_dir: PathBuf, chunks: Vec<(Vector2<i32>, Arc<RwLock<ChunkData>>)>) {
    let level = Arc::new(Level::from_root_folder(root_dir));

    level.write_chunks(chunks).await;
}

// Depends on config options from `./config`
fn bench_chunk_io(c: &mut Criterion) {
    // System temp dirs are in-memory, so we cant use temp_dir
    let root_dir = global_path!("./bench_root_tmp");
    let _ = fs::remove_dir_all(&root_dir); // delete if it exists
    fs::create_dir(&root_dir).unwrap(); // create the directory

    let async_handler = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();

    println!("Initializing data...");

    // Initial writes
    let mut chunks = Vec::new();
    let mut positions = Vec::new();
    async_handler.block_on(async {
        let (send, mut recv) = tokio::sync::mpsc::channel(10);
        // Our data dir is empty, so we're generating new chunks here
        let level = Arc::new(Level::from_root_folder(root_dir.clone()));
        tokio::spawn(async move {
            let cylindrical = Cylindrical::new(Vector2::new(0, 0), NonZeroU8::new(32).unwrap());
            let chunk_positions = cylindrical.all_chunks_within();
            level.fetch_chunks(&chunk_positions, send).await;
            level.clean_chunks(&chunk_positions).await;
        });
        while let Some((chunk, _)) = recv.recv().await {
            let pos = chunk.read().await.position;
            chunks.push((pos, chunk));
            positions.push(pos);
        }
    });

    // Sort by distance from origin to ensure a fair selection
    // when using a subset of the total chunks for the benchmarks
    chunks.sort_unstable_by_key(|chunk| chunk.0.x * chunk.0.x + chunk.0.z * chunk.0.z);
    positions.sort_unstable_by_key(|pos| pos.x * pos.x + pos.z * pos.z);

    // These test worst case: no caching done by `Level`
    // testing with 16, 64, 256 chunks
    let mut write_group = c.benchmark_group("write_chunks");
    for n_chunks in [16, 64, 256] {
        let chunks = &chunks[..n_chunks];
        println!("Testing with {} chunks", n_chunks);
        write_group.bench_with_input(
            BenchmarkId::from_parameter(n_chunks),
            &chunks,
            |b, chunks| {
                b.to_async(&async_handler)
                    .iter(|| test_writes(root_dir.clone(), chunks.to_vec()))
            },
        );
    }
    write_group.finish();

    // These test worst case: no caching done by `Level`
    // testing with 16, 64, 256 chunks
    let mut read_group = c.benchmark_group("read_chunks");
    for n_chunks in [16, 64, 256] {
        let positions = &positions[..n_chunks];
        println!("Testing with {} chunks", n_chunks);

        read_group.bench_with_input(
            BenchmarkId::from_parameter(n_chunks),
            &positions,
            |b, positions| {
                b.to_async(&async_handler)
                    .iter(|| test_reads(root_dir.clone(), positions.to_vec()))
            },
        );
    }
    read_group.finish();

    fs::remove_dir_all(&root_dir).unwrap(); // cleanup
}

criterion_group!(benches, bench_populate_noise, bench_chunk_io);
criterion_main!(benches);
