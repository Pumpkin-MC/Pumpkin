use std::sync::Arc;

use async_trait::async_trait;
use pumpkin_data::BlockState;
use pumpkin_data::noise_router::{
    END_BASE_NOISE_ROUTER, NETHER_BASE_NOISE_ROUTER, OVERWORLD_BASE_NOISE_ROUTER,
};
use pumpkin_util::math::{vector2::Vector2, vector3::Vector3};

use super::{
    biome_coords, noise::router::proto_noise_router::ProtoNoiseRouters,
    settings::gen_settings_from_dimension,
};
use crate::chunk::format::LightContainer;
use crate::generation::proto_chunk::TerrainCache;
use crate::generation::section_coords;
use crate::level::Level;
use crate::world::BlockRegistryExt;
use crate::{chunk::ChunkLight, dimension::Dimension};
use crate::{
    chunk::{
        ChunkData, ChunkSections, SubChunk,
        palette::{BiomePalette, BlockPalette},
    },
    generation::{GlobalRandomConfig, Seed, proto_chunk::PendingChunk},
};

pub trait GeneratorInit {
    fn new(seed: Seed, dimension: Dimension) -> Self;
}

#[async_trait]
pub trait WorldGenerator: Sync + Send {
    #[deprecated(note = "Use staged chunk gen instead")]
    fn generate_chunk(
        &self,
        level: &Arc<Level>,
        block_registry: &dyn BlockRegistryExt,
        at: &Vector2<i32>,
    ) -> ChunkData;

    fn new_staged_chunk(&self, at: &Vector2<i32>) -> PendingChunk;
}

pub struct VanillaGenerator {
    random_config: GlobalRandomConfig,
    base_router: ProtoNoiseRouters,
    dimension: Dimension,

    terrain_cache: TerrainCache,

    default_block: &'static BlockState,
}

impl GeneratorInit for VanillaGenerator {
    fn new(seed: Seed, dimension: Dimension) -> Self {
        let random_config = GlobalRandomConfig::new(seed.0, false);

        // TODO: The generation settings contains (part of?) the noise routers too; do we keep the separate or
        // use only the generation settings?
        let base = match dimension {
            Dimension::Overworld => OVERWORLD_BASE_NOISE_ROUTER,
            Dimension::Nether => NETHER_BASE_NOISE_ROUTER,
            Dimension::End => END_BASE_NOISE_ROUTER,
        };
        let terrain_cache = TerrainCache::from_random(&random_config);
        let generation_settings = gen_settings_from_dimension(&dimension);

        let default_block = generation_settings.default_block.get_state();
        let base_router = ProtoNoiseRouters::generate(&base, &random_config);
        Self {
            random_config,
            base_router,
            dimension,
            terrain_cache,
            default_block,
        }
    }
}

impl WorldGenerator for VanillaGenerator {
    fn generate_chunk(
        &self,
        level: &Arc<Level>,
        block_registry: &dyn BlockRegistryExt,
        at: &Vector2<i32>,
    ) -> ChunkData {
        let generation_settings = gen_settings_from_dimension(&self.dimension);

        // Use StagedChunk for type-safe generation pipeline
        PendingChunk::generate_complete_from_vanilla_generator(
            *at,
            level,
            block_registry,
            generation_settings,
            &self.random_config,
            &self.terrain_cache,
            &self.base_router,
            self.dimension,
            self.default_block,
        )
        .expect("Failed to generate chunk through staged pipeline")
    }

    fn new_staged_chunk(&self, at: &Vector2<i32>) -> PendingChunk {
        let settings = gen_settings_from_dimension(&self.dimension);
        use crate::biome::hash_seed;
        let biome_mixer_seed = hash_seed(self.random_config.seed);
        PendingChunk::new(*at, settings, self.default_block, biome_mixer_seed)
    }
}
