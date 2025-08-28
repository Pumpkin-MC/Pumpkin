use std::sync::Arc;

use async_trait::async_trait;
use pumpkin_data::BlockState;
use pumpkin_data::noise_router::{
    END_BASE_NOISE_ROUTER, NETHER_BASE_NOISE_ROUTER, OVERWORLD_BASE_NOISE_ROUTER,
};
use pumpkin_util::math::vector2::Vector2;

use super::{
    noise::router::proto_noise_router::ProtoNoiseRouters, settings::gen_settings_from_dimension,
};
use crate::dimension::Dimension;
use crate::generation::proto_chunk::TerrainCache;
use crate::level::Level;
use crate::world::BlockRegistryExt;
use crate::{
    chunk::ChunkData,
    generation::{GlobalRandomConfig, Seed, proto_chunk::StagedChunk},
};

pub trait GeneratorInit {
    fn new(seed: Seed, dimension: Dimension) -> Self;
}

#[async_trait]
pub trait WorldGenerator: Sync + Send {
    fn generate_chunk(
        &self,
        level: &Arc<Level>,
        block_registry: &dyn BlockRegistryExt,
        at: &Vector2<i32>,
    ) -> ChunkData;

    fn new_staged_chunk(&self, at: &Vector2<i32>) -> StagedChunk;
}

pub struct VanillaGenerator {
    pub random_config: GlobalRandomConfig,
    pub base_router: ProtoNoiseRouters,
    pub dimension: Dimension,

    pub terrain_cache: TerrainCache,

    pub default_block: &'static BlockState,
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
        StagedChunk::generate_complete_from_vanilla_generator(
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

    fn new_staged_chunk(&self, at: &Vector2<i32>) -> StagedChunk {
        let settings = gen_settings_from_dimension(&self.dimension);
        use crate::biome::hash_seed;
        let biome_mixer_seed = hash_seed(self.random_config.seed);
        StagedChunk::new(*at, settings, self.default_block, biome_mixer_seed)
    }
}
