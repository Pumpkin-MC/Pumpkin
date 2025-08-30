use pumpkin_data::BlockState;
use pumpkin_data::noise_router::{
    END_BASE_NOISE_ROUTER, NETHER_BASE_NOISE_ROUTER, OVERWORLD_BASE_NOISE_ROUTER,
};

use super::{
    noise::router::proto_noise_router::ProtoNoiseRouters, settings::gen_settings_from_dimension,
};
use crate::dimension::Dimension;
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
