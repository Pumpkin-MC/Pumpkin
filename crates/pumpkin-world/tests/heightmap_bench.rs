//! TEMPORARY -- development scaffolding for the light engine work, not meant to ship.
//!
//! Cost of the heightmap update on the block change path, which is what a player breaking a
//! block pays before the light engine even runs.
//!
//! `cargo test -p pumpkin-world --release --test heightmap_bench -- --nocapture`
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::print_stdout,
    clippy::panic
)]

use pumpkin_data::{Block, BlockState};
use pumpkin_world::chunk::{ChunkData, ChunkHeightmapType};

const SURFACE: i32 = 70;
const ROUNDS: usize = 400;

/// A chunk with a stone floor, a soil layer and a leaf canopy, so the three heightmaps sit at
/// different heights and a break at the canopy makes more than one of them search.
fn wooded_chunk() -> ChunkData {
    let chunk = ChunkData::empty(0, 0);
    for x in 0..16 {
        for z in 0..16 {
            for y in 0..=64 {
                chunk.set_block_absolute_y(x, y, z, Block::STONE.default_state.id);
            }
            chunk.set_block_absolute_y(x, SURFACE, z, Block::OAK_LEAVES.default_state.id);
        }
    }
    chunk
}

#[test]
fn breaking_the_canopy_updates_every_heightmap() {
    let chunk = wooded_chunk();
    let min_y = chunk.section.min_y;

    let air = Block::AIR.default_state.id;
    let leaves = Block::OAK_LEAVES.default_state.id;

    // One untimed pass, so the palettes and caches are already warm.
    for x in 0..16 {
        chunk.set_block_absolute_y(x, SURFACE, 0, air);
        chunk.set_block_absolute_y(x, SURFACE, 0, leaves);
    }

    let started = std::time::Instant::now();
    for round in 0..ROUNDS {
        let z = round % 16;
        for x in 0..16 {
            chunk.set_block_absolute_y(x, SURFACE, z, air);
            chunk.set_block_absolute_y(x, SURFACE, z, leaves);
        }
    }
    let elapsed = started.elapsed();

    let changes = (ROUNDS * 16 * 2) as u32;
    println!(
        "{changes} canopy block changes in {elapsed:?} -> {:?} per change",
        elapsed / changes,
    );

    // The column still has to be right after all that churn.
    let heightmap = chunk
        .heightmap
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    assert_eq!(
        heightmap.get(ChunkHeightmapType::WorldSurface, 0, 0, min_y),
        SURFACE
    );
    assert_eq!(
        heightmap.get(ChunkHeightmapType::MotionBlockingNoLeaves, 0, 0, min_y),
        64
    );
    let _ = BlockState::from_id(air);
}
