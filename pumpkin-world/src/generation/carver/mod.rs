mod cave;
mod mask;
#[cfg(test)]
mod test;

use std::{collections::HashMap, sync::LazyLock};

use pumpkin_data::{
    Block, BlockState, chunk_gen_settings::GenerationSettings, dimension::Dimension, tag,
};
use pumpkin_util::{
    math::{float_provider::FloatProvider, vector2::Vector2, vector3::Vector3},
    random::{RandomGenerator, get_large_feature_seed, legacy_rand::LegacyRand},
};
use serde::Deserialize;

use crate::{
    block::BlockStateCodec,
    generation::biome_coords,
    generation::{
        GlobalRandomConfig,
        biome::NoiseBiomeSampler,
        height_provider::HeightProvider,
        noise::{
            CHUNK_DIM,
            aquifer_sampler::{
                AquiferSampler, FluidLevel, FluidLevelSampler, SeaLevelAquiferSampler,
                WorldAquiferSampler,
            },
            router::{
                chunk_density_function::{
                    ChunkNoiseFunctionBuilderOptions, ChunkNoiseFunctionSampleOptions, SampleAction,
                },
                chunk_noise_router::ChunkNoiseRouter,
                proto_noise_router::ProtoNoiseRouters,
                surface_height_sampler::{
                    SurfaceHeightEstimateSampler, SurfaceHeightSamplerBuilderOptions,
                },
            },
        },
        positions::chunk_pos,
        proto_chunk::{GenerationCache, StandardChunkFluidLevelSampler, TerrainCache},
        section_coords,
        surface::{MaterialRuleContext, rule::try_apply_material_rule},
    },
};
use pumpkin_util::y_offset::YOffset;

mod ravine;

pub use mask::CarvingMask;

use self::{cave::CaveCarver, ravine::RavineCarver};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum CarvingStage {
    Air,
    Liquid,
}

#[derive(Deserialize)]
pub struct CarverConfig {
    pub probability: f32,
    pub y: HeightProvider,
    #[serde(rename = "yScale")]
    pub y_scale: FloatProvider,
    pub lava_level: YOffset,
    #[serde(default)]
    pub debug_settings: Option<CarverDebugSettings>,
    pub replaceable: String,
}

#[derive(Deserialize, Clone)]
pub struct CarverDebugSettings {
    #[serde(default, rename = "debug_mode")]
    pub debug_mode: bool,
    pub air_state: BlockStateCodec,
    pub water_state: BlockStateCodec,
    pub lava_state: BlockStateCodec,
    pub barrier_state: BlockStateCodec,
}

pub struct CarverContext<'a, 'b, T: GenerationCache> {
    pub cache: &'a mut T,
    pub chunk_pos: Vector2<i32>,
    pub carver_chunk_pos: Vector2<i32>,
    pub random: &'a mut RandomGenerator,
    pub min_y: i8,
    pub height: u16,
    pub upgrading: bool,
    pub sea_level: i32,
    pub lava_level: i32,
    pub air_mask: &'a mut CarvingMask,
    pub liquid_mask: &'a mut CarvingMask,
    pub aquifer_sampler: &'a mut CarverAquiferSampler<'b>,
    pub debug_settings: Option<&'a CarverDebugSettings>,
    pub debug_enabled: bool,
    pub nether_carver: bool,
    pub surface_rules: &'a GenerationSettings,
    pub surface_context: &'a mut MaterialRuleContext<'b>,
    pub surface_height_estimator: &'a mut SurfaceHeightEstimateSampler<'b>,
}

impl<'a, 'b, T: GenerationCache> CarverContext<'a, 'b, T> {
    fn is_masked(&self, offset_x: i32, y: i32, offset_z: i32) -> bool {
        self.air_mask.get(offset_x, y, offset_z) || self.liquid_mask.get(offset_x, y, offset_z)
    }

    fn mark_mask(&mut self, stage: CarvingStage, offset_x: i32, y: i32, offset_z: i32) {
        match stage {
            CarvingStage::Air => self.air_mask.set(offset_x, y, offset_z),
            CarvingStage::Liquid => self.liquid_mask.set(offset_x, y, offset_z),
        }
    }
}

pub struct CarverAquiferSampler<'a> {
    sampler: AquiferSampler,
    router: ChunkNoiseRouter<'a>,
    surface_height_estimator: SurfaceHeightEstimateSampler<'a>,
    sample_options: ChunkNoiseFunctionSampleOptions,
    should_schedule_fluid_update: bool,
}

struct CarverSetup<'a> {
    chunk_pos: Vector2<i32>,
    min_y: i8,
    height: u16,
    sea_level: i32,
    base_lava_level: i32,
    air_mask: CarvingMask,
    liquid_mask: CarvingMask,
    aquifer_sampler: CarverAquiferSampler<'a>,
    surface_context: MaterialRuleContext<'a>,
    surface_height_estimator: SurfaceHeightEstimateSampler<'a>,
    noise_biome_sampler: NoiseBiomeSampler<'a>,
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum CarverSelection {
    Overworld,
    Nether,
}

pub trait Carver {
    fn should_carve(&self, random: &mut RandomGenerator) -> bool;
    fn carve<T: GenerationCache>(&self, context: &mut CarverContext<'_, '_, T>);
}

#[derive(Deserialize)]
#[serde(tag = "type", content = "config")]
pub enum ConfiguredCarver {
    #[serde(rename = "minecraft:cave")]
    Cave(CaveCarver),
    #[serde(rename = "minecraft:canyon")]
    Canyon(RavineCarver),
    #[serde(rename = "minecraft:nether_cave")]
    NetherCave(CaveCarver),
}

#[derive(Deserialize)]
#[serde(untagged)]
enum BiomeCarversField {
    Single(String),
    List(Vec<String>),
}

impl Default for BiomeCarversField {
    fn default() -> Self {
        Self::List(Vec::new())
    }
}

impl BiomeCarversField {
    fn into_vec(self) -> Vec<String> {
        match self {
            Self::Single(value) => vec![value],
            Self::List(values) => values,
        }
    }
}

#[derive(Deserialize)]
struct BiomeGenerationSettings {
    #[serde(default)]
    carvers: BiomeCarversField,
}

pub static CONFIGURED_CARVERS: LazyLock<HashMap<String, ConfiguredCarver>> = LazyLock::new(|| {
    pumpkin_util::serde_json::from_str(include_str!("../../../../assets/carver.json"))
        .expect("failed to decode assets/carver.json")
});

static BIOME_CARVERS: LazyLock<HashMap<String, Vec<String>>> = LazyLock::new(|| {
    let raw: HashMap<String, BiomeGenerationSettings> =
        pumpkin_util::serde_json::from_str(include_str!("../../../../assets/biome.json"))
            .expect("failed to decode assets/biome.json");
    raw.into_iter()
        .map(|(name, settings)| (name, settings.carvers.into_vec()))
        .collect()
});

static DEFAULT_DEBUG_SETTINGS: LazyLock<CarverDebugSettings> =
    LazyLock::new(|| CarverDebugSettings {
        debug_mode: false,
        air_state: BlockStateCodec {
            name: &Block::ACACIA_BUTTON,
            properties: None,
        },
        water_state: BlockStateCodec {
            name: &Block::CANDLE,
            properties: None,
        },
        lava_state: BlockStateCodec {
            name: &Block::ORANGE_STAINED_GLASS,
            properties: None,
        },
        barrier_state: BlockStateCodec {
            name: &Block::GLASS,
            properties: None,
        },
    });

const OVERWORLD_CARVER_TAG: &str = "overworld_carver";
const NETHER_CARVER_TAG: &str = "nether_carver";

fn resolve_debug_settings(
    config: &CarverConfig,
    debug_enabled: bool,
) -> Option<&CarverDebugSettings> {
    if debug_enabled {
        Some(
            config
                .debug_settings
                .as_ref()
                .unwrap_or(&DEFAULT_DEBUG_SETTINGS),
        )
    } else {
        None
    }
}

fn is_debug_enabled(config: &CarverConfig) -> bool {
    config
        .debug_settings
        .as_ref()
        .is_some_and(|debug_settings| debug_settings.debug_mode)
}

fn should_use_carver(config: &CarverConfig, required_tag: &str) -> bool {
    config.replaceable.contains(required_tag)
}

fn is_replaceable(block_id: u16, replaceable_tag: &str) -> bool {
    let tag_name = replaceable_tag.strip_prefix("#").unwrap_or(replaceable_tag);
    tag::get_tag_ids(tag::RegistryKey::Block, tag_name)
        .map(|ids| ids.contains(&block_id))
        .unwrap_or(false)
}

fn has_fluid_state(state: &'static BlockState) -> bool {
    let block = Block::from_state_id(state.id);
    state.is_liquid() || block.is_waterlogged(state.id)
}

fn carve_block<T: GenerationCache>(
    context: &mut CarverContext<'_, '_, T>,
    pos: Vector3<i32>,
    replaceable_tag: &str,
    _is_surface: bool,
    surface_flag: &mut bool,
) -> Option<CarvingStage> {
    let current = GenerationCache::get_block_state(context.cache, &pos);
    let block_id = current.to_block_id();
    if block_id == Block::GRASS_BLOCK || block_id == Block::MYCELIUM {
        *surface_flag = true;
    }
    if !is_replaceable(block_id, replaceable_tag)
        && (!context.debug_enabled || context.nether_carver)
    {
        return None;
    }

    let debug_settings = if context.debug_enabled {
        context.debug_settings
    } else {
        None
    };
    let mut should_schedule_fluid = false;
    enum CarveKind {
        Air,
        Water,
        Lava,
    }

    let mut used_barrier_state = false;
    let (carved_state, carved_kind) = if context.nether_carver {
        if pos.y <= context.min_y as i32 + 31 {
            (Block::LAVA.default_state, CarveKind::Lava)
        } else {
            (Block::CAVE_AIR.default_state, CarveKind::Air)
        }
    } else if pos.y <= context.lava_level {
        (Block::LAVA.default_state, CarveKind::Lava)
    } else {
        let sampled_state = {
            let sampler = &mut *context.aquifer_sampler;
            sampler.sample_carve_state(&pos)
        };
        should_schedule_fluid = context.aquifer_sampler.should_schedule_fluid_update();
        match sampled_state {
            Some(state) => {
                let block = Block::from_state_id(state.id);
                let carved_kind = if block == &Block::LAVA {
                    CarveKind::Lava
                } else if block == &Block::WATER {
                    CarveKind::Water
                } else {
                    CarveKind::Air
                };
                (state, carved_kind)
            }
            None => {
                if context.debug_enabled {
                    if let Some(settings) = debug_settings {
                        used_barrier_state = true;
                        (settings.barrier_state.get_state(), CarveKind::Air)
                    } else {
                        return None;
                    }
                } else {
                    return None;
                }
            }
        }
    };

    let carved_state = if context.debug_enabled && !used_barrier_state && !context.nether_carver {
        if let Some(settings) = debug_settings {
            match carved_kind {
                CarveKind::Lava => settings.lava_state.get_state(),
                CarveKind::Water => set_waterlogged_state(settings.water_state.get_state(), true),
                CarveKind::Air => settings.air_state.get_state(),
            }
        } else {
            carved_state
        }
    } else {
        carved_state
    };

    context.cache.set_block_state(&pos, carved_state);
    if should_schedule_fluid && has_fluid_state(carved_state) {
        context.cache.mark_pos_for_postprocessing(&pos);
    }
    if *surface_flag && !context.nether_carver {
        CarverAquiferSampler::apply_top_material(
            context.cache,
            &pos,
            context.surface_rules,
            context.surface_context,
            context.surface_height_estimator,
            has_fluid_state(carved_state),
        );
    }
    Some(match carved_kind {
        CarveKind::Air => CarvingStage::Air,
        CarveKind::Water | CarveKind::Lava => CarvingStage::Liquid,
    })
}

fn set_waterlogged_state(state: &'static BlockState, waterlogged: bool) -> &'static BlockState {
    let block = Block::from_state_id(state.id);
    let Some(properties) = block.properties(state.id) else {
        return state;
    };
    let props = properties.to_props();
    let has_waterlogged = props.iter().any(|(key, _)| *key == "waterlogged");
    if !has_waterlogged {
        return state;
    }
    let mut updated_props = Vec::with_capacity(props.len());
    for (key, value) in props {
        if key == "waterlogged" {
            updated_props.push((key, if waterlogged { "true" } else { "false" }));
        } else {
            updated_props.push((key, value));
        }
    }
    let state_id = block.from_properties(&updated_props).to_state_id(block);
    BlockState::from_id(state_id)
}

fn take_carving_masks<T: GenerationCache>(cache: &mut T) -> (CarvingMask, CarvingMask) {
    let (air_mask, liquid_mask) = {
        let chunk = cache.get_center_chunk_mut();
        (
            chunk.take_carving_mask(CarvingStage::Air),
            chunk.take_carving_mask(CarvingStage::Liquid),
        )
    };
    (air_mask, liquid_mask)
}

fn chunk_biome_sampling(
    settings: &GenerationSettings,
    chunk_x: i32,
    chunk_z: i32,
) -> (i32, i32, usize) {
    let horizontal_cell_count = CHUNK_DIM / settings.shape.horizontal_cell_block_count();
    let horizontal_biome_end = crate::generation::biome_coords::from_block(
        i32::from(horizontal_cell_count) * i32::from(settings.shape.horizontal_cell_block_count()),
    );
    let start_biome_x = biome_coords::from_block(chunk_pos::start_block_x(chunk_x));
    let start_biome_z = biome_coords::from_block(chunk_pos::start_block_z(chunk_z));
    let horizontal_biome_end = horizontal_biome_end.saturating_sub(1);
    (start_biome_x, start_biome_z, horizontal_biome_end as usize)
}

fn build_surface_context<'a>(
    settings: &GenerationSettings,
    min_y: i8,
    height: u16,
    random_config: &'a GlobalRandomConfig,
    terrain_cache: &'a TerrainCache,
) -> MaterialRuleContext<'a> {
    let noise_builder =
        crate::generation::noise::router::proto_noise_router::DoublePerlinNoiseBuilder::new(
            random_config,
        );
    MaterialRuleContext::new(
        min_y,
        height,
        noise_builder,
        &random_config.base_random_deriver,
        &terrain_cache.terrain_builder,
        &terrain_cache.surface_noise,
        &terrain_cache.secondary_noise,
        settings.sea_level,
    )
}

fn build_surface_height_estimator<'a>(
    settings: &GenerationSettings,
    chunk_x: i32,
    chunk_z: i32,
    noise_router: &'a ProtoNoiseRouters,
) -> SurfaceHeightEstimateSampler<'a> {
    let (start_biome_x, start_biome_z, horizontal_biome_end) =
        chunk_biome_sampling(settings, chunk_x, chunk_z);
    let surface_config = SurfaceHeightSamplerBuilderOptions::new(
        start_biome_x,
        start_biome_z,
        horizontal_biome_end,
        settings.shape.min_y as i32,
        settings.shape.max_y() as i32,
        settings.shape.vertical_cell_block_count() as usize,
    );
    SurfaceHeightEstimateSampler::generate(&noise_router.surface_estimator, &surface_config)
}

fn build_noise_biome_sampler<'a>(
    settings: &GenerationSettings,
    chunk_x: i32,
    chunk_z: i32,
    noise_router: &'a ProtoNoiseRouters,
) -> NoiseBiomeSampler<'a> {
    let (start_biome_x, start_biome_z, horizontal_biome_end) =
        chunk_biome_sampling(settings, chunk_x, chunk_z);
    NoiseBiomeSampler::new(
        noise_router,
        start_biome_x,
        start_biome_z,
        horizontal_biome_end,
    )
}

fn build_aquifer_sampler<'a>(
    settings: &GenerationSettings,
    chunk_x: i32,
    chunk_z: i32,
    min_y: i8,
    height: u16,
    random_config: &'a GlobalRandomConfig,
    noise_router: &'a ProtoNoiseRouters,
) -> CarverAquiferSampler<'a> {
    create_aquifer_sampler(
        settings,
        chunk_x,
        chunk_z,
        min_y,
        height,
        random_config,
        noise_router,
    )
}

fn build_carver_setup<'a, T: GenerationCache>(
    cache: &mut T,
    settings: &GenerationSettings,
    chunk_x: i32,
    chunk_z: i32,
    random_config: &'a GlobalRandomConfig,
    noise_router: &'a ProtoNoiseRouters,
    terrain_cache: &'a TerrainCache,
) -> CarverSetup<'a> {
    let (min_y, height) = {
        let chunk = cache.get_center_chunk();
        (chunk.bottom_y(), chunk.height())
    };
    let chunk_pos = Vector2::new(chunk_x, chunk_z);
    let (air_mask, liquid_mask) = take_carving_masks(cache);
    let aquifer_sampler = build_aquifer_sampler(
        settings,
        chunk_x,
        chunk_z,
        min_y,
        height,
        random_config,
        noise_router,
    );
    let surface_context =
        build_surface_context(settings, min_y, height, random_config, terrain_cache);
    let surface_height_estimator =
        build_surface_height_estimator(settings, chunk_x, chunk_z, noise_router);
    let noise_biome_sampler = build_noise_biome_sampler(settings, chunk_x, chunk_z, noise_router);
    CarverSetup {
        chunk_pos,
        min_y,
        height,
        sea_level: settings.sea_level,
        base_lava_level: -54,
        air_mask,
        liquid_mask,
        aquifer_sampler,
        surface_context,
        surface_height_estimator,
        noise_biome_sampler,
    }
}

fn run_carver<'a, 'b, T: GenerationCache, C: Carver>(
    carver: &C,
    config: &'a CarverConfig,
    context: &mut CarverContext<'a, 'b, T>,
    min_y: i8,
    height: u16,
) {
    context.debug_enabled = is_debug_enabled(config);
    context.lava_level = config.lava_level.get_y(min_y as i16, height);
    context.debug_settings = resolve_debug_settings(config, context.debug_enabled);
    if carver.should_carve(context.random) {
        carver.carve(context);
    }
}

#[allow(clippy::too_many_arguments)]
fn carve_dimension<T: GenerationCache>(
    cache: &mut T,
    settings: &GenerationSettings,
    chunk_x: i32,
    chunk_z: i32,
    random_config: &GlobalRandomConfig,
    noise_router: &ProtoNoiseRouters,
    dimension: Dimension,
    selection: CarverSelection,
) -> Vec<(i32, i32)> {
    let terrain_cache = TerrainCache::from_random(random_config);
    let mut setup = build_carver_setup(
        cache,
        settings,
        chunk_x,
        chunk_z,
        random_config,
        noise_router,
        &terrain_cache,
    );
    let required_tag = match selection {
        CarverSelection::Overworld => OVERWORLD_CARVER_TAG,
        CarverSelection::Nether => NETHER_CARVER_TAG,
    };

    for dx in -8..=8 {
        for dz in -8..=8 {
            let neighbor_x = chunk_x + dx;
            let neighbor_z = chunk_z + dz;
            let neighbor_chunk_pos = Vector2::new(neighbor_x, neighbor_z);
            let biome = setup.noise_biome_sampler.biome(
                dimension,
                biome_coords::from_block(chunk_pos::start_block_x(neighbor_x)),
                0,
                biome_coords::from_block(chunk_pos::start_block_z(neighbor_z)),
            );

            let Some(carver_ids) = BIOME_CARVERS.get(biome.registry_id) else {
                continue;
            };

            for (carver_index, carver_id) in carver_ids.iter().enumerate() {
                let carver_id = carver_id.as_str();
                let carver_key = carver_id.strip_prefix("minecraft:").unwrap_or(carver_id);
                let Some(configured) = CONFIGURED_CARVERS.get(carver_key) else {
                    continue;
                };

                let carver_seed_base = random_config.seed.wrapping_add(carver_index as u64);
                let carver_seed = get_large_feature_seed(carver_seed_base, neighbor_x, neighbor_z);
                let mut random = RandomGenerator::Legacy(LegacyRand::from_seed(carver_seed));

                let upgrading = cache.get_center_chunk().is_upgrading();
                let mut context = CarverContext {
                    cache,
                    chunk_pos: setup.chunk_pos,
                    carver_chunk_pos: neighbor_chunk_pos,
                    random: &mut random,
                    min_y: setup.min_y,
                    height: setup.height,
                    upgrading,
                    sea_level: setup.sea_level,
                    lava_level: setup.base_lava_level,
                    air_mask: &mut setup.air_mask,
                    liquid_mask: &mut setup.liquid_mask,
                    aquifer_sampler: &mut setup.aquifer_sampler,
                    debug_settings: None,
                    debug_enabled: false,
                    nether_carver: selection == CarverSelection::Nether,
                    surface_rules: settings,
                    surface_context: &mut setup.surface_context,
                    surface_height_estimator: &mut setup.surface_height_estimator,
                };

                match (selection, configured) {
                    (CarverSelection::Overworld, ConfiguredCarver::Cave(carver)) => {
                        if !should_use_carver(&carver.config, required_tag) {
                            continue;
                        }
                        run_carver(
                            carver,
                            &carver.config,
                            &mut context,
                            setup.min_y,
                            setup.height,
                        );
                    }
                    (CarverSelection::Overworld, ConfiguredCarver::Canyon(carver)) => {
                        if !should_use_carver(&carver.config, required_tag) {
                            continue;
                        }
                        run_carver(
                            carver,
                            &carver.config,
                            &mut context,
                            setup.min_y,
                            setup.height,
                        );
                    }
                    (CarverSelection::Nether, ConfiguredCarver::NetherCave(carver)) => {
                        if !should_use_carver(&carver.config, required_tag) {
                            continue;
                        }
                        run_carver(
                            carver,
                            &carver.config,
                            &mut context,
                            setup.min_y,
                            setup.height,
                        );
                    }
                    _ => {}
                }
            }
        }
    }

    let carved_columns = setup.air_mask.marked_columns_union(&setup.liquid_mask);
    let CarverSetup {
        air_mask,
        liquid_mask,
        ..
    } = setup;
    let chunk = cache.get_center_chunk_mut();
    chunk.store_carving_mask(CarvingStage::Air, air_mask);
    chunk.store_carving_mask(CarvingStage::Liquid, liquid_mask);
    carved_columns
}

pub fn carve_overworld<T: GenerationCache>(
    cache: &mut T,
    settings: &GenerationSettings,
    chunk_x: i32,
    chunk_z: i32,
    random_config: &GlobalRandomConfig,
    noise_router: &ProtoNoiseRouters,
) -> Vec<(i32, i32)> {
    carve_dimension(
        cache,
        settings,
        chunk_x,
        chunk_z,
        random_config,
        noise_router,
        Dimension::OVERWORLD,
        CarverSelection::Overworld,
    )
}

pub fn carve_nether<T: GenerationCache>(
    cache: &mut T,
    settings: &GenerationSettings,
    chunk_x: i32,
    chunk_z: i32,
    random_config: &GlobalRandomConfig,
    noise_router: &ProtoNoiseRouters,
) -> Vec<(i32, i32)> {
    carve_dimension(
        cache,
        settings,
        chunk_x,
        chunk_z,
        random_config,
        noise_router,
        Dimension::THE_NETHER,
        CarverSelection::Nether,
    )
}

fn create_aquifer_sampler<'a>(
    settings: &GenerationSettings,
    chunk_x: i32,
    chunk_z: i32,
    min_y: i8,
    height: u16,
    random_config: &GlobalRandomConfig,
    noise_router: &'a ProtoNoiseRouters,
) -> CarverAquiferSampler<'a> {
    let start_x = chunk_pos::start_block_x(chunk_x);
    let start_z = chunk_pos::start_block_z(chunk_z);
    let default_fluid = Block::from_registry_key(settings.default_fluid.name)
        .unwrap_or_else(|| panic!("Unknown default fluid: {}", settings.default_fluid.name));
    let sampler = FluidLevelSampler::Chunk(StandardChunkFluidLevelSampler::new(
        FluidLevel::new(settings.sea_level, default_fluid),
        FluidLevel::new(-54, &Block::LAVA),
    ));
    let section_x = section_coords::block_to_section(start_x);
    let section_z = section_coords::block_to_section(start_z);
    let aquifer_sampler = if settings.aquifers_enabled {
        AquiferSampler::Aquifer(WorldAquiferSampler::new(
            section_x,
            section_z,
            random_config.aquifer_random_deriver(),
            min_y,
            height,
            sampler,
        ))
    } else {
        AquiferSampler::SeaLevel(SeaLevelAquiferSampler::new(sampler))
    };

    create_fluid_only_sampler(
        aquifer_sampler,
        chunk_x,
        chunk_z,
        min_y,
        height,
        settings,
        noise_router,
    )
}

fn create_fluid_only_sampler<'a>(
    aquifer_sampler: AquiferSampler,
    chunk_x: i32,
    chunk_z: i32,
    min_y: i8,
    height: u16,
    settings: &GenerationSettings,
    noise_router: &'a ProtoNoiseRouters,
) -> CarverAquiferSampler<'a> {
    let start_x = chunk_pos::start_block_x(chunk_x);
    let start_z = chunk_pos::start_block_z(chunk_z);
    let horizontal_cell_count = CHUNK_DIM / settings.shape.horizontal_cell_block_count();
    let vertical_cell_count = height as usize / settings.shape.vertical_cell_block_count() as usize;
    let horizontal_biome_end = crate::generation::biome_coords::from_block(
        i32::from(horizontal_cell_count) * i32::from(settings.shape.horizontal_cell_block_count()),
    );
    let horizontal_biome_end = horizontal_biome_end.saturating_sub(1);
    let surface_config = SurfaceHeightSamplerBuilderOptions::new(
        crate::generation::biome_coords::from_block(start_x),
        crate::generation::biome_coords::from_block(start_z),
        horizontal_biome_end as usize,
        min_y as i32,
        settings.shape.max_y() as i32,
        settings.shape.vertical_cell_block_count() as usize,
    );
    let surface_height_estimator =
        SurfaceHeightEstimateSampler::generate(&noise_router.surface_estimator, &surface_config);
    let router = ChunkNoiseRouter::generate(
        &noise_router.noise,
        &ChunkNoiseFunctionBuilderOptions::new(
            settings.shape.horizontal_cell_block_count() as usize,
            settings.shape.vertical_cell_block_count() as usize,
            vertical_cell_count,
            horizontal_cell_count as usize,
            crate::generation::biome_coords::from_block(start_x),
            crate::generation::biome_coords::from_block(start_z),
            horizontal_biome_end as usize,
        ),
    );
    let sample_options =
        ChunkNoiseFunctionSampleOptions::new(false, SampleAction::SkipCellCaches, 0, 0, 0);

    CarverAquiferSampler {
        sampler: aquifer_sampler,
        router,
        surface_height_estimator,
        sample_options,
        should_schedule_fluid_update: false,
    }
}

impl<'a> CarverAquiferSampler<'a> {
    fn should_schedule_fluid_update(&self) -> bool {
        self.should_schedule_fluid_update
    }

    fn sample_carve_state(
        &mut self,
        pos: &Vector3<i32>,
    ) -> Option<&'static pumpkin_data::BlockState> {
        self.sample_options.cache_result_unique_id =
            self.sample_options.cache_result_unique_id.wrapping_add(1);
        self.sampler.apply_with_density_and_schedule(
            &mut self.router,
            pos,
            &self.sample_options,
            &mut self.surface_height_estimator,
            0.0,
            &mut self.should_schedule_fluid_update,
        )
    }

    fn apply_top_material<T: GenerationCache>(
        cache: &mut T,
        pos: &Vector3<i32>,
        settings: &GenerationSettings,
        surface_context: &mut MaterialRuleContext<'_>,
        surface_height_estimator: &mut SurfaceHeightEstimateSampler<'_>,
        is_underwater: bool,
    ) {
        let below = Vector3::new(pos.x, pos.y - 1, pos.z);
        let below_state = GenerationCache::get_block_state(cache, &below).to_block_id();
        if below_state != Block::DIRT {
            return;
        }

        surface_context.init_horizontal(below.x, below.z);
        surface_context.init_vertical(
            1,
            1,
            below.y,
            if is_underwater { below.y + 1 } else { i32::MIN },
        );
        surface_context.biome = cache.get_biome_for_terrain_gen(below.x, below.y, below.z);
        if let Some(top_state) = try_apply_material_rule(
            &settings.surface_rule,
            cache.get_center_chunk_mut(),
            surface_context,
            surface_height_estimator,
        ) {
            cache.set_block_state(&below, top_state);
            if has_fluid_state(top_state) {
                cache.mark_pos_for_postprocessing(&below);
            }
        }
    }
}

pub fn carve_sphere<T: GenerationCache>(
    context: &mut CarverContext<'_, '_, T>,
    replaceable_tag: &str,
    center_x: f64,
    center_y: f64,
    center_z: f64,
    radius: f64,
) {
    let start_x = chunk_pos::start_block_x(context.chunk_pos.x);
    let start_z = chunk_pos::start_block_z(context.chunk_pos.y);
    let min_y = context.min_y as i32 + 1;
    let max_y =
        context.min_y as i32 + context.height as i32 - 1 - if context.upgrading { 0 } else { 7 };

    let min_x = (center_x - radius).floor() as i32;
    let max_x = (center_x + radius).ceil() as i32;
    let min_z = (center_z - radius).floor() as i32;
    let max_z = (center_z + radius).ceil() as i32;
    let min_y_s = (center_y - radius).floor() as i32;
    let max_y_s = (center_y + radius).ceil() as i32;

    for x in min_x..=max_x {
        let local_x = x - start_x;
        if !(0..16).contains(&local_x) {
            continue;
        }
        let dx = (x as f64 + 0.5 - center_x) / radius;
        let dx2 = dx * dx;
        if dx2 >= 1.0 {
            continue;
        }
        for z in min_z..=max_z {
            let local_z = z - start_z;
            if !(0..16).contains(&local_z) {
                continue;
            }
            let dz = (z as f64 + 0.5 - center_z) / radius;
            let dz2 = dz * dz;
            if dx2 + dz2 >= 1.0 {
                continue;
            }
            let mut surface_flag = false;
            for y in min_y_s..=max_y_s {
                if y <= min_y || y > max_y {
                    continue;
                }
                let dy = (y as f64 - 0.5 - center_y) / radius;
                if dx2 + dy * dy + dz2 >= 1.0 {
                    continue;
                }
                if !context.debug_enabled && context.is_masked(local_x, y, local_z) {
                    continue;
                }
                let is_surface = y >= context.sea_level - 1;
                context.mark_mask(CarvingStage::Air, local_x, y, local_z);
                if let Some(stage) = carve_block(
                    context,
                    Vector3::new(x, y, z),
                    replaceable_tag,
                    is_surface,
                    &mut surface_flag,
                ) && stage == CarvingStage::Liquid
                {
                    context.mark_mask(stage, local_x, y, local_z);
                }
            }
        }
    }
}

pub(crate) fn can_reach(
    chunk_pos: Vector2<i32>,
    x: f64,
    z: f64,
    start_step: i32,
    end_step: i32,
    thickness: f32,
) -> bool {
    let center_x = chunk_pos::start_block_x(chunk_pos.x) as f64 + 8.0;
    let center_z = chunk_pos::start_block_z(chunk_pos.y) as f64 + 8.0;
    let dx = x - center_x;
    let dz = z - center_z;
    let remaining = (end_step - start_step) as f64;
    let radius = thickness as f64 + 2.0 + 16.0;
    dx * dx + dz * dz - remaining * remaining <= radius * radius
}

#[allow(clippy::too_many_arguments)]
fn carve_ellipsoid_internal<T: GenerationCache, F: FnMut(f64, f64, f64, i32) -> bool>(
    context: &mut CarverContext<'_, '_, T>,
    replaceable_tag: &str,
    center_x: f64,
    center_y: f64,
    center_z: f64,
    horizontal_radius: f64,
    vertical_radius: f64,
    mut should_skip: F,
) {
    let start_x = chunk_pos::start_block_x(context.chunk_pos.x);
    let start_z = chunk_pos::start_block_z(context.chunk_pos.y);
    let min_y = context.min_y as i32 + 1;
    let max_y =
        context.min_y as i32 + context.height as i32 - 1 - if context.upgrading { 0 } else { 7 };

    let min_x = (center_x - horizontal_radius).floor() as i32;
    let max_x = (center_x + horizontal_radius).ceil() as i32;
    let min_z = (center_z - horizontal_radius).floor() as i32;
    let max_z = (center_z + horizontal_radius).ceil() as i32;
    let min_y_s = (center_y - vertical_radius).floor() as i32;
    let max_y_s = (center_y + vertical_radius).ceil() as i32;

    for x in min_x..=max_x {
        let local_x = x - start_x;
        if !(0..16).contains(&local_x) {
            continue;
        }
        let dx = (x as f64 + 0.5 - center_x) / horizontal_radius;
        let dx2 = dx * dx;
        if dx2 >= 1.0 {
            continue;
        }
        for z in min_z..=max_z {
            let local_z = z - start_z;
            if !(0..16).contains(&local_z) {
                continue;
            }
            let dz = (z as f64 + 0.5 - center_z) / horizontal_radius;
            let dz2 = dz * dz;
            if dx2 + dz2 >= 1.0 {
                continue;
            }
            let mut surface_flag = false;
            for y in min_y_s..=max_y_s {
                if y <= min_y || y > max_y {
                    continue;
                }
                let dy = (y as f64 - 0.5 - center_y) / vertical_radius;
                if dx2 + dy * dy + dz2 >= 1.0 {
                    continue;
                }
                if should_skip(dx, dy, dz, y) {
                    continue;
                }
                if !context.debug_enabled && context.is_masked(local_x, y, local_z) {
                    continue;
                }
                let is_surface = y >= context.sea_level - 1;
                context.mark_mask(CarvingStage::Air, local_x, y, local_z);
                if let Some(stage) = carve_block(
                    context,
                    Vector3::new(x, y, z),
                    replaceable_tag,
                    is_surface,
                    &mut surface_flag,
                ) && stage == CarvingStage::Liquid
                {
                    context.mark_mask(stage, local_x, y, local_z);
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn carve_ellipsoid<T: GenerationCache>(
    context: &mut CarverContext<'_, '_, T>,
    replaceable_tag: &str,
    center_x: f64,
    center_y: f64,
    center_z: f64,
    horizontal_radius: f64,
    vertical_radius: f64,
) {
    carve_ellipsoid_internal(
        context,
        replaceable_tag,
        center_x,
        center_y,
        center_z,
        horizontal_radius,
        vertical_radius,
        |_dx, _dy, _dz, _y| false,
    );
}

#[allow(clippy::too_many_arguments)]
pub fn carve_ellipsoid_skip<T: GenerationCache, F: FnMut(f64, f64, f64) -> bool>(
    context: &mut CarverContext<'_, '_, T>,
    replaceable_tag: &str,
    center_x: f64,
    center_y: f64,
    center_z: f64,
    horizontal_radius: f64,
    vertical_radius: f64,
    mut should_skip: F,
) {
    carve_ellipsoid_internal(
        context,
        replaceable_tag,
        center_x,
        center_y,
        center_z,
        horizontal_radius,
        vertical_radius,
        |dx, dy, dz, _y| should_skip(dx, dy, dz),
    );
}

#[allow(clippy::too_many_arguments)]
pub fn carve_ellipsoid_skip_with_y<T: GenerationCache, F: FnMut(f64, f64, f64, i32) -> bool>(
    context: &mut CarverContext<'_, '_, T>,
    replaceable_tag: &str,
    center_x: f64,
    center_y: f64,
    center_z: f64,
    horizontal_radius: f64,
    vertical_radius: f64,
    should_skip: F,
) {
    carve_ellipsoid_internal(
        context,
        replaceable_tag,
        center_x,
        center_y,
        center_z,
        horizontal_radius,
        vertical_radius,
        should_skip,
    );
}
