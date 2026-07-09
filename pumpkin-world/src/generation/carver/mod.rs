pub mod canyon;
pub mod cave;
pub mod mask;

use crate::ProtoChunk;
use crate::generation::GlobalRandomConfig;
use crate::generation::generator::VanillaGenerator;
use crate::generation::noise::aquifer_sampler::CarverAquiferSampler;
use crate::generation::noise::perlin::DoublePerlinNoiseSampler;
use crate::generation::noise::router::surface_height_sampler::{
    SurfaceHeightEstimateSampler, SurfaceHeightSamplerBuilderOptions,
};
use crate::generation::surface::rule::try_apply_material_rule;
use crate::generation::surface::terrain::SurfaceTerrainBuilder;
use crate::generation::surface::{MaterialRuleContext, steep_material_condition};
use pumpkin_data::block_state::BlockState;
use pumpkin_data::carver::{CANYON, CAVE, CAVE_EXTRA_UNDERGROUND, NETHER_CAVE};
use pumpkin_data::carver::{CarverAdditionalConfig, CarverConfig};
use pumpkin_data::chunk_gen_settings::MaterialRule;
use pumpkin_data::dimension::Dimension;
use pumpkin_data::fluid::Fluid;
use pumpkin_util::math::vector2::Vector2;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::random::{RandomGenerator, RandomImpl};

const OVERWORLD_CARVERS: [&CarverConfig; 3] = [&CAVE, &CAVE_EXTRA_UNDERGROUND, &CANYON];
const NETHER_CARVERS: [&CarverConfig; 1] = [&NETHER_CAVE];

pub struct CarverBlockIds {
    pub air: &'static BlockState,
    pub cave_air: &'static BlockState,
    pub lava: &'static BlockState,
    pub dirt: &'static BlockState,
    pub grass_block: &'static BlockState,
    pub mycelium: &'static BlockState,
}

impl Default for CarverBlockIds {
    fn default() -> Self {
        Self::new()
    }
}

impl CarverBlockIds {
    #[must_use]
    pub fn new() -> Self {
        Self {
            air: pumpkin_data::Block::AIR.default_state,
            cave_air: pumpkin_data::Block::CAVE_AIR.default_state,
            lava: pumpkin_data::Block::LAVA.default_state,
            dirt: pumpkin_data::Block::DIRT.default_state,
            grass_block: pumpkin_data::Block::GRASS_BLOCK.default_state,
            mycelium: pumpkin_data::Block::MYCELIUM.default_state,
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
    pub carver_aquifer: Option<CarverAquiferSampler<'a>>,
}

pub struct CarveRun<'a, 'b> {
    pub ctx: &'a mut CarvingContext<'b>,
    pub chunk: &'a mut ProtoChunk,
    pub ids: CarverBlockIds,
}

impl CarvingContext<'_> {
    pub fn top_material(
        &mut self,
        chunk: &mut ProtoChunk,
        x: i32,
        y: i32,
        z: i32,
        under_fluid: bool,
        steep: bool,
    ) -> Option<&'static BlockState> {
        let mut context = MaterialRuleContext::new(
            self.min_y,
            self.height,
            &self.random_config.base_random_deriver,
            self.terrain_builder,
            self.surface_noise,
            self.secondary_noise,
            self.sea_level,
        );
        context.init_horizontal(x, z);
        context.biome = chunk.get_terrain_gen_biome(x, y, z);
        context.set_steep_material_condition(steep);
        context.init_vertical(1, 1, y, if under_fluid { y + 1 } else { i32::MIN });

        try_apply_material_rule(
            self.surface_rule,
            chunk,
            &mut context,
            &mut self.surface_height_sampler,
        )
    }
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

    let carvers_to_use = carvers_for_dimension(&generator.dimension);

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
    let carver_aquifer = generator.settings.aquifers_enabled.then(|| {
        CarverAquiferSampler::new(
            chunk_x,
            chunk_z,
            &generator.base_router,
            &generator.random_config,
            generator.settings,
        )
    });

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
        carver_aquifer,
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
                let seed = get_large_feature_seed(
                    generator.random_config.seed + index as u64,
                    carver_x,
                    carver_z,
                );
                let mut carver_random =
                    new_carver_random(seed, generator.settings.legacy_random_source);

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

fn get_large_feature_seed(seed: u64, chunk_x: i32, chunk_z: i32) -> u64 {
    let mut random = pumpkin_util::random::legacy_rand::LegacyRand::from_seed(seed);
    let x_scale = random.next_i64();
    let z_scale = random.next_i64();
    let seed = seed as i64;
    let result =
        (chunk_x as i64).wrapping_mul(x_scale) ^ (chunk_z as i64).wrapping_mul(z_scale) ^ seed;
    result as u64
}

const fn new_carver_random(seed: u64, non_vanilla_random: bool) -> RandomGenerator {
    if non_vanilla_random {
        RandomGenerator::Xoroshiro(pumpkin_util::random::xoroshiro128::Xoroshiro::from_seed(
            seed,
        ))
    } else {
        RandomGenerator::Legacy(pumpkin_util::random::legacy_rand::LegacyRand::from_seed(
            seed,
        ))
    }
}

fn carvers_for_dimension(dimension: &Dimension) -> &'static [&'static CarverConfig] {
    if dimension == &Dimension::OVERWORLD {
        &OVERWORLD_CARVERS
    } else if dimension == &Dimension::THE_NETHER {
        &NETHER_CARVERS
    } else {
        &[]
    }
}

fn carve_top_material(
    run: &mut CarveRun,
    x: i32,
    carved_y: i32,
    z: i32,
    carved_state: &'static BlockState,
    has_grass: bool,
    overworld: bool,
) {
    if !overworld || !has_grass {
        return;
    }

    let below_y = carved_y - 1;
    let below_state = run.chunk.get_block_state(&Vector3::new(x, below_y, z));
    if below_state != run.ids.dirt.id {
        return;
    }

    let steep = steep_material_condition(run.chunk, x, z);
    if let Some(top_material) =
        run.ctx
            .top_material(run.chunk, x, below_y, z, carved_state.is_liquid(), steep)
    {
        run.chunk.set_block_state(x, below_y, z, top_material);

        schedule_fluid_tick_for_state(run.chunk, x, below_y, z, top_material);
    }
}

fn overworld_carve_state(
    run: &mut CarveRun,
    config: &CarverConfig,
    x: i32,
    y: i32,
    z: i32,
) -> Option<(&'static BlockState, bool)> {
    let lava_y = config
        .lava_level
        .get_y(run.chunk.bottom_y() as i16, run.chunk.height());

    if y <= lava_y {
        return Some((run.ids.lava, false));
    }

    let Some(aquifer) = run.ctx.carver_aquifer.as_mut() else {
        return Some((run.ids.air, false));
    };

    let result = aquifer.compute(&Vector3::new(x, y, z), 0.0);
    result
        .state
        .map(|state| (state, result.should_schedule_fluid_update))
}

fn place_carved_block(
    run: &mut CarveRun,
    pos: Vector3<i32>,
    state: &'static BlockState,
    should_schedule_fluid_update: bool,
    has_grass: bool,
    overworld: bool,
) {
    run.chunk.set_block_state(pos.x, pos.y, pos.z, state);

    if overworld && should_schedule_fluid_update && state.is_liquid() {
        schedule_fluid_tick_for_state(run.chunk, pos.x, pos.y, pos.z, state);
    }

    carve_top_material(run, pos.x, pos.y, pos.z, state, has_grass, overworld);
}

fn schedule_fluid_tick_for_state(
    chunk: &mut ProtoChunk,
    x: i32,
    y: i32,
    z: i32,
    state: &'static BlockState,
) {
    if state.id == pumpkin_data::Block::WATER.default_state.id {
        chunk.schedule_fluid_tick(x, y, z, &Fluid::WATER);
    } else if state.id == pumpkin_data::Block::LAVA.default_state.id {
        chunk.schedule_fluid_tick(x, y, z, &Fluid::LAVA);
    }
}

#[cfg(test)]
fn with_carve_run<F>(dimension: Dimension, test: F)
where
    F: FnOnce(&mut CarveRun<'_, '_>),
{
    with_carve_run_options(dimension, None, true, test);
}

#[cfg(test)]
fn with_carve_run_options<F>(
    dimension: Dimension,
    surface_rule: Option<&MaterialRule>,
    use_carver_aquifer: bool,
    test: F,
) where
    F: FnOnce(&mut CarveRun<'_, '_>),
{
    use crate::generation::generator::{GeneratorInit, VanillaGenerator};
    use pumpkin_util::world_seed::Seed;

    let generator = VanillaGenerator::new(Seed(42), dimension);
    let mut chunk = ProtoChunk::new(0, 0, &generator);

    let start_x = crate::generation::positions::chunk_pos::start_block_x(chunk.x);
    let start_z = crate::generation::positions::chunk_pos::start_block_z(chunk.z);
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
    let carver_aquifer = use_carver_aquifer.then(|| {
        CarverAquiferSampler::new(
            chunk.x,
            chunk.z,
            &generator.base_router,
            &generator.random_config,
            generator.settings,
        )
    });
    let mut context = CarvingContext {
        min_y: generator.dimension.min_y as i8,
        height: generator.dimension.logical_height as u16,
        random_config: &generator.random_config,
        surface_noise: &generator.terrain_cache.surface_noise,
        secondary_noise: &generator.terrain_cache.secondary_noise,
        terrain_builder: &generator.terrain_cache.terrain_builder,
        sea_level: generator.settings.sea_level,
        surface_rule: surface_rule.unwrap_or(&generator.settings.surface_rule),
        surface_height_sampler,
        carver_aquifer,
    };
    let mut run = CarveRun {
        ctx: &mut context,
        chunk: &mut chunk,
        ids: CarverBlockIds::new(),
    };

    test(&mut run);
}

#[cfg(test)]
mod tests {
    use super::*;
    use pumpkin_data::Block;
    use pumpkin_data::chunk_gen_settings::{
        BlockMaterialRule, ConditionMaterialRule, MaterialCondition, SequenceMaterialRule,
        WaterMaterialCondition,
    };

    static PODZOL_RULE: MaterialRule = MaterialRule::Block(BlockMaterialRule {
        result_state: Block::PODZOL.default_state,
    });
    static GRASS_RULE: MaterialRule = MaterialRule::Block(BlockMaterialRule {
        result_state: Block::GRASS_BLOCK.default_state,
    });
    static WATER_SENSITIVE_RULES: [MaterialRule; 2] = [
        MaterialRule::Condition(ConditionMaterialRule {
            if_true: MaterialCondition::Water(WaterMaterialCondition {
                offset: 0,
                surface_depth_multiplier: 0,
                add_stone_depth: false,
            }),
            then_run: &GRASS_RULE,
        }),
        MaterialRule::Block(BlockMaterialRule {
            result_state: Block::DIRT.default_state,
        }),
    ];
    static WATER_SENSITIVE_RULE: MaterialRule = MaterialRule::Sequence(SequenceMaterialRule {
        sequence: &WATER_SENSITIVE_RULES,
    });

    #[test]
    fn overworld_has_aquifer() {
        with_carve_run(Dimension::OVERWORLD, |run| {
            assert!(run.ctx.carver_aquifer.is_some());
        });
    }

    #[test]
    fn restores_surface() {
        with_carve_run_options(Dimension::OVERWORLD, Some(&PODZOL_RULE), false, |run| {
            let x = 4;
            let y = 70;
            let z = 5;
            run.chunk
                .set_block_state(x, y - 1, z, Block::DIRT.default_state);

            carve_top_material(run, x, y, z, Block::AIR.default_state, true, true);

            assert_eq!(
                run.chunk.get_block_state(&Vector3::new(x, y - 1, z)),
                Block::PODZOL.default_state.id,
            );
        });
    }

    #[test]
    fn skips_surface_restore() {
        with_carve_run_options(Dimension::OVERWORLD, Some(&PODZOL_RULE), false, |run| {
            let x = 4;
            let y = 70;
            let z = 5;

            run.chunk
                .set_block_state(x, y - 1, z, Block::DIRT.default_state);
            carve_top_material(run, x, y, z, Block::AIR.default_state, false, true);
            assert_eq!(
                run.chunk.get_block_state(&Vector3::new(x, y - 1, z)),
                Block::DIRT.default_state.id,
            );

            run.chunk
                .set_block_state(x, y - 1, z, Block::STONE.default_state);
            carve_top_material(run, x, y, z, Block::AIR.default_state, true, true);
            assert_eq!(
                run.chunk.get_block_state(&Vector3::new(x, y - 1, z)),
                Block::STONE.default_state.id,
            );

            run.chunk
                .set_block_state(x, y - 1, z, Block::DIRT.default_state);
            carve_top_material(run, x, y, z, Block::AIR.default_state, true, false);
            assert_eq!(
                run.chunk.get_block_state(&Vector3::new(x, y - 1, z)),
                Block::DIRT.default_state.id,
            );
        });
    }

    #[test]
    fn passes_fluid_to_rule() {
        with_carve_run_options(
            Dimension::OVERWORLD,
            Some(&WATER_SENSITIVE_RULE),
            false,
            |run| {
                let x = 6;
                let y = 70;
                let z = 7;

                let dry = run
                    .ctx
                    .top_material(run.chunk, x, y - 1, z, false, false)
                    .unwrap();
                let under_fluid = run
                    .ctx
                    .top_material(run.chunk, x, y - 1, z, true, false)
                    .unwrap();

                assert_eq!(dry.id, Block::GRASS_BLOCK.default_state.id);
                assert_eq!(under_fluid.id, Block::DIRT.default_state.id);
            },
        );
    }

    #[test]
    fn steep_matches_vanilla() {
        with_carve_run(Dimension::OVERWORLD, |run| {
            let x = 5;
            let z = 5;

            run.chunk.flat_surface_height_map = [64; crate::chunk::CHUNK_AREA];
            set_surface_height(run.chunk, x, z - 1, 60);
            set_surface_height(run.chunk, x, z + 1, 64);
            assert!(steep_material_condition(run.chunk, x, z));

            run.chunk.flat_surface_height_map = [64; crate::chunk::CHUNK_AREA];
            set_surface_height(run.chunk, x, z - 1, 64);
            set_surface_height(run.chunk, x, z + 1, 60);
            assert!(!steep_material_condition(run.chunk, x, z));

            run.chunk.flat_surface_height_map = [64; crate::chunk::CHUNK_AREA];
            set_surface_height(run.chunk, x - 1, z, 64);
            set_surface_height(run.chunk, x + 1, z, 60);
            assert!(steep_material_condition(run.chunk, x, z));

            run.chunk.flat_surface_height_map = [64; crate::chunk::CHUNK_AREA];
            set_surface_height(run.chunk, x - 1, z, 60);
            set_surface_height(run.chunk, x + 1, z, 64);
            assert!(!steep_material_condition(run.chunk, x, z));
        });
    }

    fn set_surface_height(chunk: &mut ProtoChunk, x: i32, z: i32, height: i16) {
        let index = (x & 15) as usize * 16 + (z & 15) as usize;
        chunk.flat_surface_height_map[index] = height;
    }
}
