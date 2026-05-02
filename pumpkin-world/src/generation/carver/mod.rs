pub mod canyon;
pub mod cave;
pub mod mask;

use crate::ProtoChunk;
use crate::generation::generator::VanillaGenerator;
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
    );
}

pub fn carve(chunk: &mut ProtoChunk, generator: &VanillaGenerator) {
    let mut random = RandomGenerator::Xoroshiro(
        pumpkin_util::random::xoroshiro128::Xoroshiro::from_seed(generator.random_config.seed),
    );

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

    for dx in -radius..=radius {
        for dz in -radius..=radius {
            let carver_x = chunk_x + dx;
            let carver_z = chunk_z + dz;
            let carver_chunk_pos = Vector2::new(carver_x, carver_z);

            // In vanilla, carvers are per-biome. Here we use the hardcoded list but
            // maintain the random seed logic.
            for (index, &config) in carvers_to_use.iter().enumerate() {
                let seed = get_carver_seed(
                    &mut random,
                    generator.random_config.seed + index as u64,
                    carver_x,
                    carver_z,
                );
                let mut carver_random = RandomGenerator::Xoroshiro(
                    pumpkin_util::random::xoroshiro128::Xoroshiro::from_seed(seed),
                );

                if should_carve(config, &mut carver_random) {
                    match config.additional {
                        CarverAdditionalConfig::Cave(_) | CarverAdditionalConfig::NetherCave(_) => {
                            cave_carver.carve(
                                config,
                                chunk,
                                &mut carver_random,
                                &chunk_pos,
                                &carver_chunk_pos,
                            );
                        }
                        CarverAdditionalConfig::Canyon(_) => {
                            canyon_carver.carve(
                                config,
                                chunk,
                                &mut carver_random,
                                &chunk_pos,
                                &carver_chunk_pos,
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
