//! Reproduction harness for the chunk-scheduler busy-spin.
//!
//! Requests a view-distance-sized square of chunks concurrently, the way a
//! player join does, and reports how much CPU the `Schedule` thread burned
//! while doing it.
//!
//! Run with: `cargo run --release --example join_burst -- <dir> <radius>`

// Reporting the measurement to stdout is the entire point of this example.
#![allow(clippy::print_stdout)]

use pumpkin_config::world::LevelConfig;
use pumpkin_data::dimension::Dimension;
use pumpkin_util::math::vector2::Vector2;
use pumpkin_world::dimension::into_level;
use pumpkin_world::world::WorldPortalExt;
use std::sync::Arc;
use std::time::Instant;

struct StubPortal;

impl WorldPortalExt for StubPortal {
    fn can_place_at(
        &self,
        _block: &pumpkin_data::Block,
        _state: &pumpkin_data::BlockState,
        _block_accessor: &dyn pumpkin_world::world::BlockAccessor,
        _block_pos: &pumpkin_util::math::position::BlockPos,
    ) -> bool {
        true
    }

    fn mirror(
        &self,
        block: &pumpkin_data::Block,
        state_id: pumpkin_data::BlockStateId,
        mirror: pumpkin_data::Mirror,
    ) -> &'static pumpkin_data::BlockState {
        block.mirror(state_id, mirror)
    }

    fn rotate(
        &self,
        block: &pumpkin_data::Block,
        state_id: pumpkin_data::BlockStateId,
        rotation: pumpkin_data::Rotation,
    ) -> &'static pumpkin_data::BlockState {
        block.rotate(state_id, rotation)
    }

    fn spawn_mobs_for_chunk_generation(
        &self,
        _cache: &mut dyn pumpkin_world::generation::proto_chunk::GenerationCache,
        _biome: &'static pumpkin_data::chunk::Biome,
        _chunk_x: i32,
        _chunk_z: i32,
    ) {
    }
}

/// Sums utime+stime (in clock ticks) for every thread named `name`.
fn thread_cpu_ticks(name: &str) -> u64 {
    let mut total = 0;
    let Ok(tasks) = std::fs::read_dir("/proc/self/task") else {
        return 0;
    };
    for task in tasks.flatten() {
        let Ok(stat) = std::fs::read_to_string(task.path().join("stat")) else {
            continue;
        };
        // comm is parenthesised and may contain spaces; split on the last ')'.
        let Some(close) = stat.rfind(')') else { continue };
        let Some(open) = stat.find('(') else { continue };
        if &stat[open + 1..close] != name {
            continue;
        }
        let fields: Vec<&str> = stat[close + 1..].split_whitespace().collect();
        // After comm and state, utime is field 11 and stime field 12 (1-based
        // in proc(5)); here index 11 and 12 counting state as index 0.
        if fields.len() > 12 {
            total += fields[11].parse::<u64>().unwrap_or(0);
            total += fields[12].parse::<u64>().unwrap_or(0);
        }
    }
    total
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let mut args = std::env::args().skip(1);
    let dir = args.next().expect("usage: join_burst <dir> [radius]");
    let radius: i32 = args
        .next()
        .and_then(|value| value.parse().ok())
        .unwrap_or(16);

    let level = into_level(
        Dimension::OVERWORLD,
        &LevelConfig::default(),
        dir.into(),
        42,
        None,
    );
    level
        .world_portal
        .store(Arc::new(Some(Arc::new(StubPortal) as Arc<dyn WorldPortalExt>)));

    // Let the chunk-system threads reach their idle park before measuring.
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    let side = radius * 2 + 1;
    let chunk_count = side * side;
    println!("requesting {chunk_count} chunks ({side}x{side})");

    let ticks_per_sec = 100.0; // USER_HZ
    let before = thread_cpu_ticks("Schedule");
    let start = Instant::now();

    let mut handles = Vec::with_capacity(chunk_count as usize);
    for x in -radius..=radius {
        for z in -radius..=radius {
            let level = level.clone();
            handles.push(tokio::spawn(async move {
                level.get_or_fetch_chunk(Vector2::new(x, z), |_| ()).await;
            }));
        }
    }
    for handle in handles {
        handle.await.expect("chunk fetch task panicked");
    }

    let wall = start.elapsed();
    let schedule_cpu = (thread_cpu_ticks("Schedule") - before) as f64 / ticks_per_sec;

    println!(
        "wall={:.2}s  Schedule-thread CPU={:.2}s  ({:.0}% of one core)",
        wall.as_secs_f64(),
        schedule_cpu,
        100.0 * schedule_cpu / wall.as_secs_f64()
    );

    level.shutdown().await;
}
