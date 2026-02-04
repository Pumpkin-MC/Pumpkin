use super::block_light::BlockLightEngine;
use super::sky_light::SkyLightEngine;
use crate::chunk_system::generation_cache::Cache;
use pumpkin_util::math::position::BlockPos;

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

    /// Initialize lighting for newly generated chunks.
    /// This performs full lighting calculations including sky light conversion.
    pub fn initialize_light(&mut self, cache: &mut Cache) {
        self.sky_light.convert_light(cache);
        self.block_light.propagate_light(cache);
        self.sky_light.propagate_light(cache);
        
        // Validate block light to fix any ghost lights from generation
        self.block_light.validate_light(cache);
        
        // Sky light validation disabled (convert_light and propagate_light produce correct results)
        // Validation with decrease queue can cause harsh lighting artifacts during generation
        // self.sky_light.validate_light(cache); ... re-enable if issues arise
    }

    /// Update lighting when a block changes during gameplay.
    /// 
    /// This should be called when:
    /// - A light-emitting block is placed or broken (torch, glowstone, etc.)
    /// - An opaque block is placed or broken (affects light propagation)
    /// - Any block state change that affects lighting
    pub fn update_block_light(&mut self, cache: &mut Cache, pos: BlockPos, old_luminance: u8, new_luminance: u8) {
        use super::storage::{get_block_light, set_block_light};
        
        // If the light source was removed or reduced, queue decrease
        if old_luminance > new_luminance {
            let current_light = get_block_light(cache, pos);
            if current_light > 0 {
                self.block_light.decrease_queue.push_back((pos, current_light));
                set_block_light(cache, pos, 0);
            }
        }
        
        // If a new light source was added or increased, queue increase
        if new_luminance > 0 {
            set_block_light(cache, pos, new_luminance);
            if self.block_light.visited.insert(pos) {
                self.block_light.queue.push_back(pos);
            }
        }
    }

    /// Process all queued lighting updates.
    /// 
    /// Call this after one or more update_block_light() calls to propagate
    /// the lighting changes through the world.
    pub fn run_light_updates(&mut self, cache: &mut Cache) {
        // Process decrease queue first (remove old light)
        if !self.block_light.decrease_queue.is_empty() {
            self.block_light.process_decrease_queue(cache);
        }
        
        // Process increase queue (propagate new light)
        if !self.block_light.queue.is_empty() {
            self.block_light.propagate_light(cache);
        }
        
        // TODO: Add sky light updates when blocks change height/opacity
    }
}

impl Default for LightEngine {
    fn default() -> Self {
        Self::new()
    }
}
