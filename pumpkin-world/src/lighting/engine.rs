use crate::chunk_system::generation_cache::Cache;
use super::block_light::BlockLightEngine;
use super::sky_light::SkyLightEngine;

pub struct LightEngine {
    block_light: BlockLightEngine,
    sky_light: SkyLightEngine,
}

impl LightEngine {
    pub fn new() -> Self {
        Self {
            block_light: BlockLightEngine::new(),
            sky_light: SkyLightEngine::new(),
        }
    }

    pub fn initialize_light(&mut self, cache: &mut Cache) {
        self.sky_light.convert_light(cache);
        self.block_light.propagate_light(cache);
        self.sky_light.propagate_light(cache);
    }
}
