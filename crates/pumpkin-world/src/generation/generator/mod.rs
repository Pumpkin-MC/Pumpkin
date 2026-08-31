use pumpkin_data::BlockState;
use pumpkin_data::chunk_gen_settings::GenerationSettings;
use pumpkin_data::dimension::Dimension;
use pumpkin_data::noise_router::{
    END_BASE_NOISE_ROUTER, NETHER_BASE_NOISE_ROUTER, OVERWORLD_BASE_NOISE_ROUTER,
};

use super::noise::router::proto_noise_router::ProtoNoiseRouters;
use crate::generation::proto_chunk::TerrainCache;
use crate::generation::{GlobalRandomConfig, Seed};

pub mod biome_finder;
pub mod structure_finder;

pub trait GeneratorInit {
    fn new(seed: Seed, dimension: Dimension) -> Self;
}

use pumpkin_data::structures::{StructurePlacementCalculator, StructureSet};
use rustc_hash::FxHashMap;

use std::sync::Arc;

use crate::chunk_system::StagedChunkEnum;
use crate::generation::proto_chunk::ProtoChunk;

pub mod flat;

#[derive(Clone, Debug)]
pub struct FlatLayer {
    pub block: String,
    pub height: i32,
}

pub trait CustomChunkGenerator: Send + Sync {
    fn dimension(&self) -> &Dimension;
    fn seed(&self) -> u64;
    fn default_block(&self) -> &'static BlockState {
        pumpkin_data::Block::AIR.default_state
    }
    fn biome_mixer_seed(&self) -> i64 {
        0
    }
    fn global_structure_cache(
        &self,
    ) -> Option<&crate::generation::structure::placement::GlobalStructureCache> {
        None
    }

    fn step_to_biomes(&self, chunk: &mut ProtoChunk) {
        chunk.stage = StagedChunkEnum::Biomes;
    }

    fn step_to_noise(&self, chunk: &mut ProtoChunk) {
        chunk.stage = StagedChunkEnum::Noise;
    }

    fn step_to_surface(&self, chunk: &mut ProtoChunk) {
        chunk.stage = StagedChunkEnum::Surface;
    }

    fn step_to_carvers(&self, chunk: &mut ProtoChunk) {
        chunk.stage = StagedChunkEnum::Carvers;
    }

    fn step_to_features(
        &self,
        cache: &mut crate::chunk_system::generation_cache::Cache,
        _block_registry: &dyn crate::world::WorldPortalExt,
    ) {
        let mid = ((cache.size * cache.size) >> 1) as usize;
        cache.chunks[mid].get_proto_chunk_mut().stage = StagedChunkEnum::Features;
    }

    fn set_structure_starts(&self, _chunk: &mut ProtoChunk) {}
    fn set_structure_references(&self, _chunk: &mut ProtoChunk) {}
}

pub enum WorldGenerator {
    Noise(Box<VanillaGenerator>),
    Flat(flat::FlatGenerator),
    Custom(Arc<dyn CustomChunkGenerator>),
}

impl WorldGenerator {
    #[must_use]
    pub fn dimension(&self) -> &Dimension {
        match self {
            Self::Noise(noise_gen) => &noise_gen.dimension,
            Self::Flat(flat_gen) => &flat_gen.dimension,
            Self::Custom(custom_gen) => custom_gen.dimension(),
        }
    }

    #[must_use]
    pub fn seed(&self) -> u64 {
        match self {
            Self::Noise(noise_gen) => noise_gen.random_config.seed,
            Self::Flat(flat_gen) => flat_gen.seed,
            Self::Custom(custom_gen) => custom_gen.seed(),
        }
    }

    #[must_use]
    pub fn global_structure_cache(
        &self,
    ) -> Option<&crate::generation::structure::placement::GlobalStructureCache> {
        match self {
            Self::Noise(noise_gen) => Some(&noise_gen.global_structure_cache),
            Self::Flat(_) => None,
            Self::Custom(custom_gen) => custom_gen.global_structure_cache(),
        }
    }

    #[must_use]
    pub fn find_spawn_position(&self) -> pumpkin_util::math::position::BlockPos {
        match self {
            Self::Noise(noise_gen) => noise_gen.find_spawn_position(),
            _ => pumpkin_util::math::position::BlockPos::ZERO,
        }
    }
}

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

impl VanillaGenerator {
    #[must_use]
    pub fn find_spawn_position(&self) -> pumpkin_util::math::position::BlockPos {
        if self.settings.spawn_target.is_empty() {
            return pumpkin_util::math::position::BlockPos::ZERO;
        }
        let options = crate::generation::noise::router::multi_noise_sampler::MultiNoiseSamplerBuilderOptions::new(1, 1, 1);
        let mut sampler =
            crate::generation::noise::router::multi_noise_sampler::MultiNoiseSampler::generate(
                &self.base_router.multi_noise,
                &options,
            );
        crate::biome::position_finder::SpawnFinder::find_spawn_position(
            self.settings.spawn_target,
            &mut sampler,
        )
    }
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
            tracing::error!("Unsupported dimension for noise router: {:?}", dimension);
            OVERWORLD_BASE_NOISE_ROUTER
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
