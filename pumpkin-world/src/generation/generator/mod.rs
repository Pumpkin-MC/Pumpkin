use pumpkin_data::BlockState;
use pumpkin_data::chunk_gen_settings::GenerationSettings;
use pumpkin_data::dimension::Dimension;
use pumpkin_data::noise_router::{
    END_BASE_NOISE_ROUTER, NETHER_BASE_NOISE_ROUTER, OVERWORLD_BASE_NOISE_ROUTER,
};

use super::noise::router::proto_noise_router::ProtoNoiseRouters;
use crate::generation::proto_chunk::TerrainCache;
use crate::generation::{GlobalRandomConfig, Seed};

pub mod structure_finder;

pub trait GeneratorInit {
    fn new(seed: Seed, dimension: Dimension) -> Self;
}

use pumpkin_data::structures::{StructurePlacementCalculator, StructureSet};
use rustc_hash::FxHashMap;

pub struct VanillaGenerator {
    pub random_config: GlobalRandomConfig,
    pub base_router: ProtoNoiseRouters,
    pub dimension: Dimension,
    pub settings: &'static GenerationSettings,
    pub biome_mixer_seed: i64,

    pub terrain_cache: TerrainCache,

    pub default_block: &'static BlockState,

    pub global_structure_cache: crate::generation::structure::placement::GlobalStructureCache,
    pub structure_calculator: StructurePlacementCalculator,
    pub structure_allowed_biomes: FxHashMap<usize, Vec<u16>>,
}

impl GeneratorInit for VanillaGenerator {
    fn new(seed: Seed, dimension: Dimension) -> Self {
        let settings = GenerationSettings::from_dimension(&dimension);
        let random_config = GlobalRandomConfig::new(seed.0, settings.legacy_random_source);

        // TODO: The generation settings contains (part of?) the noise routers too; do we keep the separate or
        // use only the generation settings?
        let base = if dimension == Dimension::OVERWORLD {
            OVERWORLD_BASE_NOISE_ROUTER
        } else if dimension == Dimension::THE_NETHER {
            NETHER_BASE_NOISE_ROUTER
        } else if dimension == Dimension::THE_END {
            END_BASE_NOISE_ROUTER
        } else {
            unreachable!()
        };
        let terrain_cache = TerrainCache::from_random(&random_config);

        let default_block = settings.default_block;
        let base_router = ProtoNoiseRouters::generate(&base, &random_config);
        let biome_mixer_seed = crate::biome::hash_seed(seed.0);

        let mut structure_allowed_biomes = FxHashMap::default();
        for (i, set) in StructureSet::ALL.iter().enumerate() {
            structure_allowed_biomes.insert(
                i,
                crate::generation::proto_chunk::ProtoChunk::get_allowed_biomes(set),
            );
        }

        Self {
            random_config,
            base_router,
            dimension,
            settings,
            biome_mixer_seed,
            terrain_cache,
            default_block,
            global_structure_cache:
                crate::generation::structure::placement::GlobalStructureCache::new(),
            structure_calculator: StructurePlacementCalculator::new(seed.0 as i64),
            structure_allowed_biomes,
        }
    }
}
