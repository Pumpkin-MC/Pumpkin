use pumpkin_data::BlockState;
use pumpkin_data::chunk_gen_settings::GenerationSettings;
use pumpkin_data::dimension::Dimension;
use pumpkin_data::noise_router::{
    END_BASE_NOISE_ROUTER, NETHER_BASE_NOISE_ROUTER, OVERWORLD_BASE_NOISE_ROUTER,
};

use super::noise::router::proto_noise_router::ProtoNoiseRouters;
use crate::block::to_state_from_blueprint;
use crate::generation::proto_chunk::TerrainCache;
use crate::generation::{GlobalRandomConfig, Seed};

pub trait GeneratorInit {
    fn new(seed: Seed, dimension: Dimension) -> Self;
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
        let generation_settings = GenerationSettings::from_dimension(&dimension);

        let default_block = to_state_from_blueprint(&generation_settings.default_block);
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
