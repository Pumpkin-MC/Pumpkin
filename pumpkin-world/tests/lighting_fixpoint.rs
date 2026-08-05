//! Checks that the light engine's output satisfies its own propagation rule.
//!
//! After lighting finishes, no cell should be darker than what a neighbour can
//! give it. Walking every cell of the window and re-applying
//! `LightProvider::propagate_level` is a cheap way to state that, and it does
//! not depend on knowing what the right answer is — only that the engine agrees
//! with itself.
//!
//! This is what catches the class of bug where propagation stops early. It
//! found one: `propagate` used to test its `visited` set before comparing
//! levels, and the seeding passes put their own cells in that set, so every
//! seeded cell was frozen at its seeded value and no brighter neighbour could
//! raise it. Sky light could not enter water at a shoreline, leaving cells at 1
//! directly beside open air at 15 — about 2000 of the centre chunk's 98304
//! cells, off by as much as 13 levels.

#![allow(clippy::print_stdout)]

use pumpkin_config::lighting::LightingEngineConfig;
use pumpkin_data::BlockDirection;
use pumpkin_data::BlockStateId;
use pumpkin_data::dimension::Dimension;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::world_seed::Seed;
use pumpkin_world::ProtoChunk;
use pumpkin_world::chunk_system::{Cache, Chunk, StagedChunkEnum};
use pumpkin_world::generation::get_world_gen;
use pumpkin_world::generation::height_limit::HeightLimitView;
use pumpkin_world::generation::proto_chunk::GenerationCache;
use pumpkin_world::lighting::storage::{get_block_light, get_sky_light};
use pumpkin_world::world::WorldPortalExt;

struct BlockRegistry;
impl WorldPortalExt for BlockRegistry {
    fn can_place_at(
        &self,
        _block: &pumpkin_data::Block,
        _state: &pumpkin_data::BlockState,
        _block_accessor: &dyn pumpkin_world::world::BlockAccessor,
        _block_pos: &BlockPos,
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
        _cache: &mut dyn GenerationCache,
        _biome: &'static pumpkin_data::chunk::Biome,
        _chunk_x: i32,
        _chunk_z: i32,
    ) {
    }
}

/// Generate a cache through the whole pipeline and run the lighting stage on it.
fn lit_cache(seed: u64) -> Cache {
    let world_gen = get_world_gen(
        Seed(seed),
        Dimension::OVERWORLD,
        false,
        Vec::new(),
        String::new(),
    );
    let block_registry = BlockRegistry;

    let radius = StagedChunkEnum::Lighting.get_direct_radius();
    let mut cache = Cache::new(-radius, -radius, radius * 2 + 1);
    for dx in -radius..=radius {
        for dz in -radius..=radius {
            cache
                .chunks
                .push(Chunk::Proto(Box::new(ProtoChunk::new(dx, dz, &world_gen))));
        }
    }

    for stage in [
        StagedChunkEnum::Biomes,
        StagedChunkEnum::StructureStart,
        StagedChunkEnum::StructureReferences,
        StagedChunkEnum::Noise,
        StagedChunkEnum::Surface,
        StagedChunkEnum::Carvers,
        StagedChunkEnum::Features,
        StagedChunkEnum::Lighting,
    ] {
        cache.advance(
            stage,
            &world_gen,
            &block_registry,
            &LightingEngineConfig::Default,
        );
    }

    cache
}

// The two rules, matching `LightProvider` in lighting/engine.rs. `opacity` is
// the opacity of the *destination* cell, and `dir` points from the source to
// the destination.
fn propagate_block(current: u8, opacity: u8, _dir: BlockDirection) -> u8 {
    current.saturating_sub(opacity.max(1))
}

fn propagate_sky(current: u8, opacity: u8, dir: BlockDirection) -> u8 {
    if current == 15 && dir == BlockDirection::Down && opacity == 0 {
        return 15;
    }
    current.saturating_sub(opacity.max(1))
}

#[derive(Default)]
struct Report {
    /// Violations whose cell lies in the centre chunk — the only chunk whose
    /// light is kept from this cache.
    in_centre: u64,
    /// Violations anywhere in the window, including the outer ring.
    total: u64,
    worst_deficit: u8,
    samples: Vec<String>,
}

/// Walk every cell of the window and check that no neighbour could raise it.
fn audit(
    cache: &Cache,
    name: &str,
    get: fn(&Cache, BlockPos) -> u8,
    propagate: fn(u8, u8, BlockDirection) -> u8,
) -> Report {
    let min_y = cache.bottom_y() as i32;
    let max_y = min_y + cache.height() as i32;
    let min_x = cache.x * 16;
    let max_x = (cache.x + cache.size) * 16;
    let min_z = cache.z * 16;
    let max_z = (cache.z + cache.size) * 16;

    let mut report = Report::default();

    for y in min_y..max_y {
        for z in min_z..max_z {
            for x in min_x..max_x {
                let pos = Vector3::new(x, y, z);
                let here = get(cache, BlockPos(pos));
                let opacity = cache.get_block_state(&pos).to_state().opacity;

                for dir in BlockDirection::all() {
                    // The source is the cell that would propagate *into* here
                    // along `dir`, so it sits on the opposite side.
                    let offset = dir.to_offset();
                    let src = Vector3::new(x - offset.x, y - offset.y, z - offset.z);
                    if src.y < min_y
                        || src.y >= max_y
                        || src.x < min_x
                        || src.x >= max_x
                        || src.z < min_z
                        || src.z >= max_z
                    {
                        continue;
                    }

                    let src_light = get(cache, BlockPos(src));
                    let would_give = propagate(src_light, opacity, dir);
                    if would_give <= here {
                        continue;
                    }

                    report.total += 1;
                    report.worst_deficit = report.worst_deficit.max(would_give - here);

                    let in_centre = (0..16).contains(&x) && (0..16).contains(&z);
                    if in_centre {
                        report.in_centre += 1;
                        if report.samples.len() < 10 {
                            report.samples.push(format!(
                                "  {name}: ({x},{y},{z}) has {here}, but ({},{},{}) at \
                                 {src_light} propagating {dir:?} into opacity {opacity} would \
                                 give {would_give}",
                                src.x, src.y, src.z
                            ));
                        }
                    }
                }
            }
        }
    }

    report
}

#[test]
fn lighting_output_satisfies_its_own_rule() {
    for seed in [42, 1, 7, 12345, 99999] {
        let cache = lit_cache(seed);

        let sky = audit(&cache, "sky", get_sky_light, propagate_sky);
        let block = audit(&cache, "block", get_block_light, propagate_block);

        for (name, report) in [("sky", &sky), ("block", &block)] {
            println!(
                "seed {seed:>6} {name:>5}: {} violations in the centre chunk, {} in the whole \
                 window (worst deficit {})",
                report.in_centre, report.total, report.worst_deficit
            );
            for sample in &report.samples {
                println!("{sample}");
            }
        }

        // Violations in the outer ring of the window are expected and harmless:
        // the seeding passes only cover an 18x18 column band around the centre
        // chunk, so everything past it is deliberately under-lit and never read
        // back. What has to hold is the centre chunk, the only one this cache
        // keeps.
        assert_eq!(
            (sky.in_centre, block.in_centre),
            (0, 0),
            "seed {seed}: lighting left cells in the centre chunk darker than a neighbour \
             would make them, so propagation stopped short"
        );
    }
}
