//! TEMPORARY -- development scaffolding for the light engine work, not meant to ship.
//!
//! What generated chunks actually cost in resident memory, and how much of that is light.
//!
//! Generates chunks through the real pipeline, keeps every one of them alive the way a loaded
//! world does, and reports process RSS alongside the light bytes it can account for. Run it
//! with and without the section granular sky fill to get the A/B.
//!
//! `PUMPKIN_MEMORY_CHUNKS=500 cargo test -p pumpkin-world --release --test lighting_memory -- --nocapture`
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::print_stdout,
    clippy::print_stderr,
    clippy::panic
)]

use pumpkin_data::BlockStateId;
use pumpkin_data::dimension::Dimension;
use pumpkin_util::world_seed::Seed;
use pumpkin_world::chunk::format::LightContainer;
use pumpkin_world::chunk_system::{Chunk, StagedChunkEnum, generate_single_chunk};
use pumpkin_world::generation::get_world_gen;
use pumpkin_world::world::WorldPortalExt;
use std::sync::Arc;

const SEED: Seed = Seed(42);
const CHUNKS_ENV: &str = "PUMPKIN_MEMORY_CHUNKS";
const DEFAULT_CHUNKS: usize = 400;

struct BlockRegistry;
impl WorldPortalExt for BlockRegistry {
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
        state_id: BlockStateId,
        mirror: pumpkin_data::Mirror,
    ) -> &'static pumpkin_data::BlockState {
        block.mirror(state_id, mirror)
    }

    fn rotate(
        &self,
        block: &pumpkin_data::Block,
        state_id: BlockStateId,
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

/// Resident set size in bytes, straight from the kernel.
fn rss_bytes() -> u64 {
    let statm = std::fs::read_to_string("/proc/self/statm").expect("linux");
    let pages: u64 = statm
        .split_whitespace()
        .nth(1)
        .expect("resident field")
        .parse()
        .expect("number");
    pages * 4096
}

#[derive(Default)]
struct LightBytes {
    full: u64,
    empty: u64,
}

impl LightBytes {
    fn add(&mut self, containers: &[LightContainer]) {
        for container in containers {
            match container {
                LightContainer::Full(data) => self.full += data.len() as u64,
                LightContainer::Empty(_) => self.empty += 1,
            }
        }
    }
}

#[test]
fn generated_chunks_report_their_light_footprint() {
    let count = std::env::var(CHUNKS_ENV)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_CHUNKS);

    let world_gen = get_world_gen(SEED, Dimension::OVERWORLD, false, Vec::new(), String::new());
    let registry = Arc::new(BlockRegistry);

    // Touch the generator's lazy tables before the baseline, so their allocation is not
    // counted as chunk memory.
    let _ = generate_single_chunk(
        &world_gen,
        registry.as_ref(),
        10_000,
        10_000,
        StagedChunkEnum::Full,
    );
    let baseline = rss_bytes();

    let side = (count as f64).sqrt().ceil() as i32;
    let mut held = Vec::with_capacity(count);
    for index in 0..count as i32 {
        let chunk = generate_single_chunk(
            &world_gen,
            registry.as_ref(),
            index % side,
            index / side,
            StagedChunkEnum::Full,
        );
        held.push(chunk);
    }

    let after = rss_bytes();

    let mut sky = LightBytes::default();
    let mut block = LightBytes::default();
    for chunk in &held {
        let Chunk::Level(level) = chunk else {
            panic!("a fully generated chunk should be a level chunk");
        };
        let light = level
            .light_engine
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        sky.add(&light.sky_light);
        block.add(&light.block_light);
    }

    let n = count as f64;
    let light_total = sky.full + block.full;
    println!(
        "{count} chunks held\n\
         RSS: {:.1} MiB baseline -> {:.1} MiB, {:.1} KiB per chunk\n\
         light arrays: {:.1} MiB total, {:.1} KiB per chunk ({:.0}% of the growth)\n\
         sky:   {:.1} KiB/chunk in arrays, {:.1} uniform sections/chunk\n\
         block: {:.1} KiB/chunk in arrays, {:.1} uniform sections/chunk",
        baseline as f64 / (1024.0 * 1024.0),
        after as f64 / (1024.0 * 1024.0),
        (after.saturating_sub(baseline)) as f64 / n / 1024.0,
        light_total as f64 / (1024.0 * 1024.0),
        light_total as f64 / n / 1024.0,
        light_total as f64 / (after.saturating_sub(baseline)).max(1) as f64 * 100.0,
        sky.full as f64 / n / 1024.0,
        sky.empty as f64 / n,
        block.full as f64 / n / 1024.0,
        block.empty as f64 / n,
    );

    // Keeps the chunks alive until after the measurement.
    assert_eq!(held.len(), count);
}
