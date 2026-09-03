//! TEMPORARY -- development scaffolding for the light engine work, not meant to ship.
//!
//! Cost of the *runtime* light engine, the path a player breaking or placing a block takes.
//!
//! No bench scenario reaches this code: the harness only knows `console`, `bots`, `stop_bots`,
//! `scatter`, `teleport_rounds` and `mark`, so its bots never modify a block and the runtime
//! engine sits idle in every published run. These cases stand in for it.
//!
//! `cargo test -p pumpkin-world --release --test runtime_light_bench -- --nocapture`
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::print_stdout,
    clippy::panic
)]

use pumpkin_config::world::LevelConfig;
use pumpkin_data::dimension::Dimension;
use pumpkin_data::{Block, BlockStateId};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector2::Vector2;
use pumpkin_world::chunk::ChunkData;
use pumpkin_world::chunk::format::LightContainer;
use pumpkin_world::level::Level;
use pumpkin_world::lighting::DynamicLightEngine;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tempfile::TempDir;

const SECTIONS: usize = 24;
const MIN_Y: i32 = -64;
const HEIGHT: i32 = SECTIONS as i32 * 16;
const SURFACE: i32 = 60;
/// Roof of the room carved under the surface, where the light work happens.
const ROOM_TOP: i32 = 40;
const ROOM_BOTTOM: i32 = 34;

struct World {
    level: Arc<Level>,
    engine: DynamicLightEngine,
    _dir: TempDir,
}

impl World {
    /// Four loaded chunks of solid stone with a room carved out of each, so a light source
    /// inside has somewhere to flood and a chunk border to cross.
    fn new() -> Self {
        let dir = TempDir::new().unwrap();
        let level = Level::from_root_folder(
            &LevelConfig::default(),
            dir.path().to_path_buf(),
            42,
            Dimension::OVERWORLD,
        );

        for (cx, cz) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
            let chunk = Self::build_chunk(cx, cz);
            level.loaded_chunks.insert(Vector2::new(cx, cz), chunk);
        }

        Self {
            level,
            engine: DynamicLightEngine::new(),
            _dir: dir,
        }
    }

    fn build_chunk(cx: i32, cz: i32) -> Arc<ChunkData> {
        let chunk = ChunkData::empty(cx, cz);
        let mut updates = Vec::new();
        for x in 0..16usize {
            for z in 0..16usize {
                for y in 0..=SURFACE {
                    let air = (ROOM_BOTTOM..=ROOM_TOP).contains(&y);
                    updates.push((
                        x,
                        y,
                        z,
                        if air {
                            Block::AIR.default_state.id
                        } else {
                            Block::STONE.default_state.id
                        },
                    ));
                }
            }
        }
        chunk.set_blocks_batch(updates);
        *chunk
            .heightmap
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = chunk.calculate_heightmap();

        let mut light = chunk
            .light_engine
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        light.sky_light = (0..SECTIONS)
            .map(|_| LightContainer::new_empty(0))
            .collect();
        light.block_light = (0..SECTIONS)
            .map(|_| LightContainer::new_empty(0))
            .collect();
        for x in 0..16usize {
            for z in 0..16usize {
                let mut y = MIN_Y + HEIGHT - 1;
                while y >= MIN_Y {
                    if chunk
                        .section
                        .get_block_absolute_y(x, y, z)
                        .is_some_and(|id| id != Block::AIR.default_state.id)
                    {
                        break;
                    }
                    let relative = (y - MIN_Y) as usize;
                    light.sky_light[relative / 16].set(x, relative % 16, z, 15);
                    y -= 1;
                }
            }
        }
        drop(light);

        Arc::new(chunk)
    }

    fn chunk_at(&self, cx: i32, cz: i32) -> Arc<ChunkData> {
        self.level
            .loaded_chunks
            .get(&Vector2::new(cx, cz))
            .expect("chunk was loaded")
            .clone()
    }

    /// The full player-facing path: write the block, then tell the engine.
    fn set_block(&self, pos: BlockPos, id: BlockStateId) {
        let chunk = self.chunk_at(pos.0.x >> 4, pos.0.z >> 4);
        chunk.set_block_absolute_y(
            (pos.0.x & 15) as usize,
            pos.0.y,
            (pos.0.z & 15) as usize,
            id,
        );
        self.engine.update_lighting_at(&self.level, pos);
    }

    fn settle(&self) {
        let converged = (0..4000).any(|_| !self.engine.drain_queued(&self.level).leftover);
        assert!(converged, "light updates did not converge");
    }
}

fn report(name: &str, elapsed: Duration, ops: u32) {
    println!("{name}: {elapsed:?} over {ops} ops -> {:?} per op", elapsed / ops);
}

/// A torch going in and out, the single most ordinary light update there is.
#[tokio::test]
async fn placing_and_breaking_a_light_source() {
    let world = World::new();
    let glowstone = Block::GLOWSTONE.default_state.id;
    let air = Block::AIR.default_state.id;
    let pos = BlockPos::new(8, ROOM_BOTTOM + 2, 8);

    world.set_block(pos, glowstone);
    world.settle();
    world.set_block(pos, air);
    world.settle();

    const ROUNDS: u32 = 60;
    let started = Instant::now();
    for _ in 0..ROUNDS {
        world.set_block(pos, glowstone);
        world.settle();
        world.set_block(pos, air);
        world.settle();
    }
    report("light source place+break", started.elapsed(), ROUNDS * 2);
}

/// Many blocks changed before a single drain -- an explosion or a piston, and the case
/// vanilla's `checkBlock` set is built for: it collapses repeats of one position, which
/// Pumpkin's `update_lighting_at` does eagerly per call instead.
#[tokio::test]
async fn a_burst_of_changes_before_one_drain() {
    let world = World::new();
    let stone = Block::STONE.default_state.id;
    let air = Block::AIR.default_state.id;

    let wall: Vec<BlockPos> = (0..16)
        .flat_map(|x| (ROOM_BOTTOM..=ROOM_TOP).map(move |y| BlockPos::new(x, y, 8)))
        .collect();

    world.settle();

    const ROUNDS: u32 = 12;
    let started = Instant::now();
    for _ in 0..ROUNDS {
        for pos in &wall {
            world.set_block(*pos, stone);
        }
        world.settle();
        for pos in &wall {
            world.set_block(*pos, air);
        }
        world.settle();
    }
    let ops = ROUNDS * 2 * wall.len() as u32;
    report("burst wall build+clear", started.elapsed(), ops);
}

/// The same position touched repeatedly in one batch. Vanilla checks it once per
/// `runLightUpdates`; Pumpkin re-checks on every call, so this is the dedup's headroom.
#[tokio::test]
async fn one_position_touched_many_times_before_a_drain() {
    let world = World::new();
    let glowstone = Block::GLOWSTONE.default_state.id;
    let air = Block::AIR.default_state.id;
    let pos = BlockPos::new(8, ROOM_BOTTOM + 2, 8);

    world.settle();

    const ROUNDS: u32 = 40;
    const REPEATS: u32 = 16;
    let started = Instant::now();
    for _ in 0..ROUNDS {
        for i in 0..REPEATS {
            world.set_block(pos, if i % 2 == 0 { glowstone } else { air });
        }
        world.settle();
    }
    report(
        "same position re-touched",
        started.elapsed(),
        ROUNDS * REPEATS,
    );
}
