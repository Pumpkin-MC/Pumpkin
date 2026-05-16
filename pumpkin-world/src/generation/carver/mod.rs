pub mod canyon;
pub mod cave;
pub mod mask;

use crate::ProtoChunk;
use crate::generation::generator::VanillaGenerator;
use pumpkin_data::Block;
use pumpkin_data::carver::{CANYON, CAVE, CAVE_EXTRA_UNDERGROUND, NETHER_CAVE};
use pumpkin_data::carver::{CarverAdditionalConfig, CarverConfig};
use pumpkin_util::math::vector2::Vector2;
use pumpkin_util::random::{RandomGenerator, RandomImpl, get_carver_seed};

pub trait Carver {
    fn carve(
        &self,
        config: &CarverConfig,
        chunk: &mut ProtoChunk,
        random: &mut RandomGenerator,
        chunk_pos: &Vector2<i32>,
        carver_chunk_pos: &Vector2<i32>,
        legacy_random_source: bool,
        sea_level: i32,
    );
}

pub fn carve(chunk: &mut ProtoChunk, generator: &VanillaGenerator) {
    // Vanilla applyCarvers uses a range of 8 chunks (17x17 area)
    let radius = 8;
    let chunk_x = chunk.x;
    let chunk_z = chunk.z;
    let chunk_pos = Vector2::new(chunk_x, chunk_z);

    let overworld_carvers = [&CAVE, &CAVE_EXTRA_UNDERGROUND, &CANYON];
    let nether_carvers = [&NETHER_CAVE];

    let carvers_to_use = if generator.dimension == pumpkin_data::dimension::Dimension::OVERWORLD {
        &overworld_carvers[..]
    } else if generator.dimension == pumpkin_data::dimension::Dimension::THE_NETHER {
        &nether_carvers[..]
    } else {
        &[]
    };

    let cave_carver = cave::CaveCarver;
    let canyon_carver = canyon::CanyonCarver;
    let sea_level = generator.settings.sea_level;

    for dx in -radius..=radius {
        for dz in -radius..=radius {
            let carver_x = chunk_x + dx;
            let carver_z = chunk_z + dz;
            let carver_chunk_pos = Vector2::new(carver_x, carver_z);

            // In vanilla, carvers are per-biome. Here we use the hardcoded list but
            // maintain the random seed logic.
            for (index, &config) in carvers_to_use.iter().enumerate() {
                let seed = get_carver_seed(
                    generator.random_config.seed + index as u64,
                    carver_x,
                    carver_z,
                );
                let mut carver_random = if generator.settings.legacy_random_source {
                    RandomGenerator::Legacy(
                        pumpkin_util::random::legacy_rand::LegacyRand::from_seed(seed),
                    )
                } else {
                    RandomGenerator::Xoroshiro(
                        pumpkin_util::random::xoroshiro128::Xoroshiro::from_seed(seed),
                    )
                };

                if should_carve(config, &mut carver_random) {
                    match config.additional {
                        CarverAdditionalConfig::Cave(_) | CarverAdditionalConfig::NetherCave(_) => {
                            cave_carver.carve(
                                config,
                                chunk,
                                &mut carver_random,
                                &chunk_pos,
                                &carver_chunk_pos,
                                generator.settings.legacy_random_source,
                                sea_level,
                            );
                        }
                        CarverAdditionalConfig::Canyon(_) => {
                            canyon_carver.carve(
                                config,
                                chunk,
                                &mut carver_random,
                                &chunk_pos,
                                &carver_chunk_pos,
                                generator.settings.legacy_random_source,
                                sea_level,
                            );
                        }
                    }
                }
            }
        }
    }
}

fn should_carve(config: &CarverConfig, random: &mut RandomGenerator) -> bool {
    random.next_f32() <= config.probability
}

/// What block to place at a carved position. Vanilla `WorldCarver.getCarveState`:
/// lava below lava_y, water if the column is under an ocean and y < sea_level,
/// otherwise air.
pub fn carve_block_state(
    y: i32,
    lava_y: i32,
    sea_level: i32,
    column_below_water: bool,
) -> &'static pumpkin_data::block_state::BlockState {
    let id = if y <= lava_y {
        Block::LAVA.default_state.id
    } else if column_below_water && y < sea_level {
        Block::WATER.default_state.id
    } else {
        Block::AIR.default_state.id
    };
    pumpkin_data::block_state::BlockState::from_id(id)
}

/// What the aquifer materialized at the top of this column. Extend with new
/// variants (e.g. `Lava`) when richer aquifer behavior is implemented.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Aquifer {
    /// No aquifer fluid at this column's surface — dry land.
    None,
    /// Ocean/lake column — water at `sea_level - 1`.
    Water,
}

/// Probe the column at `(world_x, world_z)` for its aquifer fluid.
/// Returns `None` if (x,z) is outside the chunk's XZ bounds (the caller
/// has no block data for that column).
pub fn column_aquifer(
    chunk: &ProtoChunk,
    world_x: i32,
    world_z: i32,
    sea_level: i32,
) -> Option<Aquifer> {
    let cx = chunk.x << 4;
    let cz = chunk.z << 4;
    if !(cx..cx + 16).contains(&world_x) || !(cz..cz + 16).contains(&world_z) {
        return None;
    }
    let local_y = (sea_level - 1) - chunk.bottom_y() as i32;
    if local_y < 0 || local_y >= chunk.height() as i32 {
        return Some(Aquifer::None);
    }
    let sid = chunk.get_block_state_raw(world_x & 15, local_y, world_z & 15);
    Some(if Block::from_state_id(sid).id == Block::WATER.id {
        Aquifer::Water
    } else {
        Aquifer::None
    })
}

/// True if any 4-neighbor column has a *different* aquifer kind. Carvers
/// skip these to leave a stone barrier (approximates vanilla's per-cell
/// barrier-density check). Neighbors outside the chunk count as matching.
pub fn column_is_aquifer_boundary(
    chunk: &ProtoChunk,
    world_x: i32,
    world_z: i32,
    sea_level: i32,
) -> bool {
    let Some(here) = column_aquifer(chunk, world_x, world_z, sea_level) else {
        return false;
    };
    [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().any(|(dx, dz)| {
        column_aquifer(chunk, world_x + dx, world_z + dz, sea_level).is_some_and(|n| n != here)
    })
}
