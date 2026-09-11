pub mod canyon;
pub mod cave;
pub mod mask;

use crate::ProtoChunk;
use crate::biome::BiomeSupplier;
use crate::generation::GlobalRandomConfig;
use crate::generation::generator::VanillaGenerator;
use crate::generation::noise::aquifer_sampler::CarverAquiferSampler;
use crate::generation::noise::perlin::DoublePerlinNoiseSampler;
use crate::generation::noise::router::multi_noise_sampler::MultiNoiseSampler;
use crate::generation::noise::router::surface_height_sampler::{
    SurfaceHeightEstimateSampler, SurfaceHeightSamplerBuilderOptions,
};
use crate::generation::surface::rule::try_apply_material_rule;
use crate::generation::surface::terrain::SurfaceTerrainBuilder;
use crate::generation::surface::{MaterialRuleContext, steep_material_condition};
use crate::generation::{biome_coords, section_coords};
use pumpkin_data::block_state::BlockState;
use pumpkin_data::carver::{CarverAdditionalConfig, CarverConfig};
#[cfg(test)]
use pumpkin_data::dimension::Dimension;
use pumpkin_data::fluid::Fluid;
use pumpkin_data::material_rule::MaterialRule;
use pumpkin_util::math::vector2::Vector2;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::random::{RandomGenerator, RandomImpl};

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
    pub default_block: &'static BlockState,
    pub default_fluid: &'static BlockState,
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

pub trait CarverOutput {
    fn carve(&mut self, x: usize, y: i32, z: usize);
    fn min_y(&self) -> i32;
    fn max_y(&self) -> i32;
}

#[must_use]
pub fn can_reach(
    chunk_pos: &Vector2<i32>,
    x: f64,
    z: f64,
    current_step: i32,
    total_steps: i32,
    thickness: f32,
) -> bool {
    let x_mid = (chunk_pos.x << 4) as f64 + 8.0;
    let z_mid = (chunk_pos.y << 4) as f64 + 8.0;
    let xd = x - x_mid;
    let zd = z - z_mid;
    let remaining = (total_steps - current_step) as f64;
    let rr = (thickness + 2.0 + 16.0) as f64;
    (xd * xd + zd * zd) - (remaining * remaining) <= rr * rr
}

#[allow(clippy::too_many_arguments)]
pub fn carve_ellipsoid<O, S>(
    chunk_pos: &Vector2<i32>,
    x: f64,
    y: f64,
    z: f64,
    horizontal_radius: f64,
    vertical_radius: f64,
    output: &mut O,
    skip_checker: S,
) where
    O: CarverOutput + ?Sized,
    S: Fn(f64, f64, f64, i32) -> bool,
{
    let center_x = (chunk_pos.x << 4) as f64 + 8.0;
    let center_z = (chunk_pos.y << 4) as f64 + 8.0;
    let max_delta = 16.0 + horizontal_radius * 2.0;

    if (x - center_x).abs() > max_delta || (z - center_z).abs() > max_delta {
        return;
    }

    let chunk_min_x = chunk_pos.x << 4;
    let chunk_min_z = chunk_pos.y << 4;

    let min_x_index = (((x - horizontal_radius).floor() as i32 - chunk_min_x) - 1).max(0) as usize;
    let max_x_index = ((x + horizontal_radius).floor() as i32 - chunk_min_x).clamp(0, 15) as usize;
    let min_y = ((y - vertical_radius).floor() as i32 - 1).max(output.min_y());
    let max_y = ((y + vertical_radius).floor() as i32 + 1).min(output.max_y());
    let min_z_index = (((z - horizontal_radius).floor() as i32 - chunk_min_z) - 1).max(0) as usize;
    let max_z_index = ((z + horizontal_radius).floor() as i32 - chunk_min_z).clamp(0, 15) as usize;

    for x_index in min_x_index..=max_x_index {
        let world_x = chunk_min_x + x_index as i32;
        let xd = (world_x as f64 + 0.5 - x) / horizontal_radius;

        for z_index in min_z_index..=max_z_index {
            let world_z = chunk_min_z + z_index as i32;
            let zd = (world_z as f64 + 0.5 - z) / horizontal_radius;

            if xd * xd + zd * zd < 1.0 {
                for world_y in (min_y + 1..=max_y).rev() {
                    let yd = (world_y as f64 - 0.5 - y) / vertical_radius;
                    if !skip_checker(xd, yd, zd, world_y) {
                        output.carve(x_index, world_y, z_index);
                    }
                }
            }
        }
    }
}

pub trait Carver {
    #[allow(clippy::too_many_arguments)]
    fn carve(
        &self,
        config: &CarverConfig,
        output: &mut dyn CarverOutput,
        random: &mut RandomGenerator,
        chunk_pos: &Vector2<i32>,
        carver_chunk_pos: &Vector2<i32>,
        min_gen_y: i8,
        gen_depth: u16,
        legacy_random_source: bool,
    );
}

#[allow(clippy::too_many_lines)]
pub fn carve(chunk: &mut ProtoChunk, generator: &VanillaGenerator) {
    // Vanilla applyCarvers uses a range of 8 chunks (17x17 area)
    let radius = 8;
    let chunk_x = chunk.x;
    let chunk_z = chunk.z;
    let chunk_pos = Vector2::new(chunk_x, chunk_z);

    let supplier: &dyn BiomeSupplier = &generator.biome_supplier;
    let mut multi_noise_sampler = MultiNoiseSampler::generate(&generator.base_router.multi_noise);

    let generation_shape = &generator.settings.shape;
    let min_gen_y = generation_shape.min_y;
    let gen_depth = generation_shape.height;
    let protected_blocks_on_top = 7;
    let min_y = min_gen_y as i32 + 1;
    let max_y = (min_gen_y as i32 + gen_depth as i32 - 1) - protected_blocks_on_top;
    let mut mask = mask::CarvingMask::new(min_y, max_y);

    let cave_carver = cave::CaveCarver;
    let canyon_carver = canyon::CanyonCarver;

    for dx in -radius..=radius {
        for dz in -radius..=radius {
            let carver_x = chunk_x + dx;
            let carver_z = chunk_z + dz;
            let carver_chunk_pos = Vector2::new(carver_x, carver_z);

            let carver_biome = if dx == 0 && dz == 0 {
                chunk.get_biome(0, 0, 0)
            } else {
                supplier.biome(
                    biome_coords::from_block(section_coords::section_to_block(carver_x)),
                    0,
                    biome_coords::from_block(section_coords::section_to_block(carver_z)),
                    &mut multi_noise_sampler,
                )
            };

            for (index, &config) in carver_biome.carvers.iter().enumerate() {
                let seed = get_large_feature_seed(
                    generator.random_config.seed + index as u64,
                    carver_x,
                    carver_z,
                );
                let mut carver_random =
                    new_carver_random(seed, generator.settings.legacy_random_source);

                if should_carve(config, &mut carver_random) {
                    match config.additional {
                        CarverAdditionalConfig::Cave(_) => {
                            cave_carver.carve(
                                config,
                                &mut mask,
                                &mut carver_random,
                                &chunk_pos,
                                &carver_chunk_pos,
                                min_gen_y,
                                gen_depth,
                                generator.settings.legacy_random_source,
                            );
                        }
                        CarverAdditionalConfig::Canyon(_) => {
                            canyon_carver.carve(
                                config,
                                &mut mask,
                                &mut carver_random,
                                &chunk_pos,
                                &carver_chunk_pos,
                                min_gen_y,
                                gen_depth,
                                generator.settings.legacy_random_source,
                            );
                        }
                    }
                }
            }
        }
    }

    if !mask.is_empty() {
        let surface_config = SurfaceHeightSamplerBuilderOptions::new(
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
            default_block: generator.settings.default_block,
            default_fluid: generator.settings.default_fluid,
            surface_rule: generator.surface_rule,
            surface_height_sampler,
            carver_aquifer,
        };

        let mut run = CarveRun {
            ctx: &mut context,
            chunk,
            ids: CarverBlockIds::new(),
        };

        apply_carving_mask(&mut run, &mask);
    }
}

fn apply_carving_mask(run: &mut CarveRun, mask: &mask::CarvingMask) {
    let chunk_x = run.chunk.x;
    let chunk_z = run.chunk.z;
    let overworld = run.ctx.carver_aquifer.is_some();
    let bedrock_id = pumpkin_data::Block::BEDROCK.default_state.id;
    let grass_block_id = pumpkin_data::Block::GRASS_BLOCK.default_state.id;
    let mycelium_id = pumpkin_data::Block::MYCELIUM.default_state.id;

    mask.visit(|x, z, bottom_y, top_y| {
        let mut has_grass = false;
        let world_x = (chunk_x << 4) + x as i32;
        let world_z = (chunk_z << 4) + z as i32;
        for world_y in (bottom_y..=top_y).rev() {
            let current_state_id = run
                .chunk
                .get_block_state(&Vector3::new(world_x, world_y, world_z));
            if current_state_id == bedrock_id {
                continue;
            }
            if current_state_id == grass_block_id || current_state_id == mycelium_id {
                has_grass = true;
            }
            if let Some((state, should_schedule_fluid_update)) =
                overworld_carve_state(run, world_x, world_y, world_z)
            {
                place_carved_block(
                    run,
                    Vector3::new(world_x, world_y, world_z),
                    state,
                    should_schedule_fluid_update,
                    has_grass,
                    overworld,
                );
            }
        }
    });
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
    x: i32,
    y: i32,
    z: i32,
) -> Option<(&'static BlockState, bool)> {
    if let Some(aquifer) = run.ctx.carver_aquifer.as_mut() {
        let result = aquifer.compute(&Vector3::new(x, y, z), 0.0);
        result
            .state
            .map(|state| (state, result.should_schedule_fluid_update))
    } else if y < run.ctx.sea_level {
        Some((run.ctx.default_fluid, false))
    } else {
        Some((run.ids.air, false))
    }
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
    use crate::generation::generator::{GeneratorInit, VanillaGenerator, WorldGenerator};
    use pumpkin_util::world_seed::Seed;

    let world_gen = WorldGenerator::Noise(Box::new(VanillaGenerator::new(Seed(42), dimension)));
    let WorldGenerator::Noise(generator) = &world_gen else {
        unreachable!()
    };
    let mut chunk = ProtoChunk::new(0, 0, &world_gen);

    let generation_shape = &generator.settings.shape;
    let surface_config = SurfaceHeightSamplerBuilderOptions::new(
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
        default_block: generator.settings.default_block,
        default_fluid: generator.settings.default_fluid,
        surface_rule: surface_rule.unwrap_or(generator.surface_rule),
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
    use pumpkin_data::material_rule::{
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

    #[test]
    fn biome_carvers_driven_by_data() {
        use pumpkin_data::biome::Biome;
        use pumpkin_data::carver::{CANYON, CAVE, CAVE_EXTRA_UNDERGROUND, NETHER_CAVE};

        assert_eq!(Biome::PLAINS.carvers.len(), 3);
        assert!(std::ptr::eq(Biome::PLAINS.carvers[0], &CAVE));
        assert!(std::ptr::eq(
            Biome::PLAINS.carvers[1],
            &CAVE_EXTRA_UNDERGROUND
        ));
        assert!(std::ptr::eq(Biome::PLAINS.carvers[2], &CANYON));

        assert_eq!(Biome::NETHER_WASTES.carvers.len(), 1);
        assert!(std::ptr::eq(Biome::NETHER_WASTES.carvers[0], &NETHER_CAVE));

        assert!(Biome::THE_END.carvers.is_empty());
    }

    #[test]
    fn apply_carving_mask_skips_bedrock() {
        with_carve_run(Dimension::OVERWORLD, |run| {
            let bedrock = Block::BEDROCK.default_state;
            let stone = Block::STONE.default_state;
            run.chunk.set_block_state(5, 10, 5, bedrock);
            run.chunk.set_block_state(5, 11, 5, stone);

            let mut mask = mask::CarvingMask::new(-64, 320);
            mask.set(5, 10, 5);
            mask.set(5, 11, 5);

            apply_carving_mask(run, &mask);

            // Bedrock must not be carved
            assert_eq!(
                run.chunk.get_block_state(&Vector3::new(5, 10, 5)),
                bedrock.id
            );
            // Stone must be carved
            assert_ne!(run.chunk.get_block_state(&Vector3::new(5, 11, 5)), stone.id);
        });
    }
}
