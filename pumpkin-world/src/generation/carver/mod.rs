pub mod canyon;
pub mod cave;
pub mod mask;

use crate::ProtoChunk;
use crate::generation::generator::VanillaGenerator;
use crate::generation::noise::perlin::DoublePerlinNoiseSampler;
use crate::generation::noise::router::surface_height_sampler::{
    SurfaceHeightEstimateSampler, SurfaceHeightSamplerBuilderOptions,
};
use crate::generation::surface::terrain::SurfaceTerrainBuilder;
use crate::generation::GlobalRandomConfig;
use pumpkin_data::block_state::BlockState;
use pumpkin_data::carver::{CANYON, CAVE, CAVE_EXTRA_UNDERGROUND, NETHER_CAVE};
use pumpkin_data::carver::{CarverAdditionalConfig, CarverConfig};
use pumpkin_data::chunk_gen_settings::MaterialRule;
use pumpkin_util::math::vector2::Vector2;
use pumpkin_util::random::{RandomGenerator, RandomImpl, get_carver_seed};

pub struct CarverBlockIds {
    pub air: &'static BlockState,
    pub cave_air: &'static BlockState,
    pub lava: &'static BlockState,
    pub dirt: &'static BlockState,
    pub grass_block: &'static BlockState,
    pub mycelium: &'static BlockState,
}

impl CarverBlockIds {
    #[must_use]
    pub fn new() -> Self {
        Self {
            air: &pumpkin_data::Block::AIR.default_state,
            cave_air: &pumpkin_data::Block::CAVE_AIR.default_state,
            lava: &pumpkin_data::Block::LAVA.default_state,
            dirt: &pumpkin_data::Block::DIRT.default_state,
            grass_block: &pumpkin_data::Block::GRASS_BLOCK.default_state,
            mycelium: &pumpkin_data::Block::MYCELIUM.default_state,
        }
    }
}

pub struct CarvingContext<'a> {
    pub min_y: i8,
    pub height: u16,
    pub random_config: &'a GlobalRandomConfig,
    pub surface_noise: &'a DoublePerlinNoiseSampler,
    pub secondary_noise: &'a DoublePerlinNoiseSampler,
    pub terrain_builder: &'a SurfaceTerrainBuilder,
    pub sea_level: i32,
    pub surface_rule: &'a MaterialRule,
    pub surface_height_sampler: SurfaceHeightEstimateSampler<'a>,
}

pub struct CarveRun<'a, 'b> {
    pub ctx: &'a mut CarvingContext<'b>,
    pub chunk: &'a mut ProtoChunk,
    pub ids: CarverBlockIds,
}

pub trait Carver {
    fn carve(
        &self,
        config: &CarverConfig,
        run: &mut CarveRun,
        random: &mut RandomGenerator,
        chunk_pos: &Vector2<i32>,
        carver_chunk_pos: &Vector2<i32>,
        legacy_random_source: bool,
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

    let start_x = crate::generation::positions::chunk_pos::start_block_x(chunk_x);
    let start_z = crate::generation::positions::chunk_pos::start_block_z(chunk_z);
    let generation_shape = &generator.settings.shape;
    let horizontal_cell_count = 16 / generation_shape.horizontal_cell_block_count();

    let horizontal_biome_end = crate::generation::biome_coords::from_block(
        horizontal_cell_count as i32 * generation_shape.horizontal_cell_block_count() as i32,
    );
    let surface_config = SurfaceHeightSamplerBuilderOptions::new(
        crate::generation::biome_coords::from_block(start_x),
        crate::generation::biome_coords::from_block(start_z),
        horizontal_biome_end as usize,
        generation_shape.min_y as i32,
        generation_shape.max_y() as i32,
        generation_shape.vertical_cell_block_count() as usize,
    );
    let surface_height_sampler = SurfaceHeightEstimateSampler::generate(
        &generator.base_router.surface_estimator,
        &surface_config,
    );

    let mut context = CarvingContext {
        min_y: generator.dimension.min_y as i8,
        height: generator.dimension.logical_height as u16,
        random_config: &generator.random_config,
        surface_noise: &generator.terrain_cache.surface_noise,
        secondary_noise: &generator.terrain_cache.secondary_noise,
        terrain_builder: &generator.terrain_cache.terrain_builder,
        sea_level: generator.settings.sea_level,
        surface_rule: &generator.settings.surface_rule,
        surface_height_sampler,
    };

    let mut run = CarveRun {
        ctx: &mut context,
        chunk,
        ids: CarverBlockIds::new(),
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
                                &mut run,
                                &mut carver_random,
                                &chunk_pos,
                                &carver_chunk_pos,
                                generator.settings.legacy_random_source,
                            );
                        }
                        CarverAdditionalConfig::Canyon(_) => {
                            canyon_carver.carve(
                                config,
                                &mut run,
                                &mut carver_random,
                                &chunk_pos,
                                &carver_chunk_pos,
                                generator.settings.legacy_random_source,
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
