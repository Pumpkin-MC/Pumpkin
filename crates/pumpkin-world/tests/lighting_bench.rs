//! Quick sanity-check timing comparison for the `DynamicLightEngine` runtime overhaul.
//!
//! `cargo test --release -p pumpkin-world --test lighting_bench -- --nocapture --ignored`
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::print_stdout)]

use pumpkin_config::world::LevelConfig;
use pumpkin_data::dimension::Dimension;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector2::Vector2;
use pumpkin_world::chunk::ChunkData;
use pumpkin_world::chunk::format::LightContainer;
use pumpkin_world::level::Level;
use pumpkin_world::lighting::DynamicLightEngine;
use std::sync::Arc;
use tempfile::TempDir;

const SECTION_COUNT: usize = 24;
const MIN_Y: i32 = -64;

fn make_level() -> Arc<Level> {
    let temp_dir = TempDir::new().unwrap();
    let config = LevelConfig::default();
    Level::from_root_folder(
        &config,
        temp_dir.path().to_path_buf(),
        42,
        Dimension::OVERWORLD,
    )
}

/// One fully-loaded, fully dark chunk at the given position.
fn insert_chunk_at(level: &Arc<Level>, cx: i32, cz: i32) {
    let chunk = ChunkData::empty(cx, cz);
    let mut light = chunk
        .light_engine
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    light.sky_light = (0..SECTION_COUNT)
        .map(|_| LightContainer::new_empty(0))
        .collect();
    light.block_light = (0..SECTION_COUNT)
        .map(|_| LightContainer::new_empty(0))
        .collect();
    drop(light);

    level
        .loaded_chunks
        .insert(Vector2::new(cx, cz), Arc::new(chunk));
}

/// Positions inside one chunk, so a producer thread can own whole chunks.
fn positions_in_chunk(cx: i32, n: usize) -> Vec<BlockPos> {
    let mut out = Vec::with_capacity(n);
    'outer: for y in 0..64 {
        for x in 0..16 {
            for z in 0..16 {
                if out.len() >= n {
                    break 'outer;
                }
                out.push(BlockPos::new(cx * 16 + x, MIN_Y + y, z));
            }
        }
    }
    out
}

/// Builds a single fully-loaded chunk at (0,0) with light storage sized to
/// match its section count (the default `ChunkData::empty` light storage is
/// zero-length, which isn't representative of a real loaded chunk).
fn insert_test_chunk(level: &Arc<Level>) {
    let chunk = ChunkData::empty(0, 0);
    let mut light = chunk
        .light_engine
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    // Start fully dark so the workload below actually has propagation work to
    // do (a chunk that is already fully sky-lit makes every check a no-op,
    // which would make the two integration patterns look identical).
    light.sky_light = (0..SECTION_COUNT)
        .map(|_| LightContainer::new_empty(0))
        .collect();
    light.block_light = (0..SECTION_COUNT)
        .map(|_| LightContainer::new_empty(0))
        .collect();
    drop(light);

    level
        .loaded_chunks
        .insert(Vector2::new(0, 0), Arc::new(chunk));
}

/// A grid of positions within the single loaded chunk (x,z in 0..16, several y
/// layers), representative of a bulk light-affecting event.
fn workload_positions(n: usize) -> Vec<BlockPos> {
    let mut out = Vec::with_capacity(n);
    let mut i = 0usize;
    'outer: for y in 0..64 {
        for x in 0..16 {
            for z in 0..16 {
                if i >= n {
                    break 'outer;
                }
                out.push(BlockPos::new(x, MIN_Y + y, z));
                i += 1;
            }
        }
    }
    out
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "manual perf sanity check, not part of the normal test suite"]
async fn lighting_bulk_update_old_vs_new_integration_pattern() {
    const N: usize = 4000;
    let positions = workload_positions(N);

    // "Old style": check + immediate full drain after every single update.
    let old_elapsed = {
        let level = make_level();
        insert_test_chunk(&level);
        let engine = DynamicLightEngine::new();
        let start = std::time::Instant::now();
        for pos in &positions {
            engine.update_lighting_at(&level, *pos);
            // Drain fully to convergence immediately, mimicking the old
            // inline perform_* calls (budget effectively unbounded).
            for _ in 0..1000 {
                let stats = engine.drain_queued(&level);
                if !stats.leftover {
                    break;
                }
            }
        }
        start.elapsed()
    };

    // "New style": enqueue everything, drain once at the end (one tick's
    // worth of budgeted work, repeated until convergence to be fair about
    // total work done).
    let new_elapsed = {
        let level = make_level();
        insert_test_chunk(&level);
        let engine = DynamicLightEngine::new();
        let start = std::time::Instant::now();
        for pos in &positions {
            engine.update_lighting_at(&level, *pos);
        }
        for _ in 0..1000 {
            let stats = engine.drain_queued(&level);
            if !stats.leftover {
                break;
            }
        }
        start.elapsed()
    };

    println!(
        "lighting bulk update ({N} positions): old-style(check+drain per call)={old_elapsed:?} new-style(enqueue all, drain once)={new_elapsed:?}"
    );
}

/// Does the light engine scale with the `par_iter` chunk ticking that feeds it?
///
/// The drain is reported apart: serialised either way, and the part a dedicated light
/// thread would take off the tick.
#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
#[ignore = "manual perf sanity check, not part of the normal test suite"]
async fn lighting_producer_scaling() {
    const CHUNKS: usize = 8;
    const PER_CHUNK: usize = 1000;

    for threads in [1usize, 2, 4, 8] {
        let level = make_level();
        for c in 0..CHUNKS {
            insert_chunk_at(&level, c as i32, 0);
        }
        let engine = Arc::new(DynamicLightEngine::new());
        let chunks_per_thread = CHUNKS / threads;

        let start = std::time::Instant::now();
        std::thread::scope(|scope| {
            for t in 0..threads {
                let level = level.clone();
                let engine = engine.clone();
                scope.spawn(move || {
                    for c in (t * chunks_per_thread)..((t + 1) * chunks_per_thread) {
                        for pos in positions_in_chunk(c as i32, PER_CHUNK) {
                            engine.update_lighting_at(&level, pos);
                        }
                    }
                });
            }
        });
        let enqueue = start.elapsed();

        let start = std::time::Instant::now();
        for _ in 0..1000 {
            if !engine.drain_queued(&level).leftover {
                break;
            }
        }
        let drain = start.elapsed();

        println!(
            "threads={threads} chunks={CHUNKS} per_chunk={PER_CHUNK} \
             enqueue(check, lock-free)={enqueue:?} drain(flood, serialised)={drain:?}"
        );
    }
}
