use crate::chunk_system::chunk_state::Chunk;
use crate::chunk_system::generation_cache::Cache;
use crate::generation::height_limit::HeightLimitView;
use crate::generation::proto_chunk::GenerationCache;
use pumpkin_util::math::position::BlockPos;

/// Fast light accessor that caches chunk lookups
/// This dramatically reduces the overhead of repeated get/set calls
pub struct FastLightCache<'a> {
    cache: &'a mut Cache,
    // Cache the last accessed chunk to avoid repeated lookups
    last_chunk_x: i32,
    last_chunk_z: i32,
    last_chunk_idx: Option<usize>,
}

impl<'a> FastLightCache<'a> {
    pub fn new(cache: &'a mut Cache) -> Self {
        Self {
            cache,
            last_chunk_x: i32::MIN,
            last_chunk_z: i32::MIN,
            last_chunk_idx: None,
        }
    }

    #[inline(always)]
    fn get_chunk_index_cached(&mut self, chunk_x: i32, chunk_z: i32) -> Option<usize> {
        // Check cache first
        if chunk_x == self.last_chunk_x && chunk_z == self.last_chunk_z {
            return self.last_chunk_idx;
        }

        // Cache miss - calculate
        let rel_x = chunk_x - self.cache.x;
        let rel_z = chunk_z - self.cache.z;
        
        if rel_x < 0 || rel_x >= self.cache.size || rel_z < 0 || rel_z >= self.cache.size {
            self.last_chunk_idx = None;
            return None;
        }

        let idx = (rel_x * self.cache.size + rel_z) as usize;
        
        // Update cache
        self.last_chunk_x = chunk_x;
        self.last_chunk_z = chunk_z;
        self.last_chunk_idx = Some(idx);
        
        Some(idx)
    }

    #[inline(always)]
    fn get_section_y(&self, pos_y: i32) -> Option<usize> {
        let bottom = self.cache.bottom_y() as i32;
        if pos_y < bottom {
            return None;
        }
        let section = ((pos_y - bottom) >> 4) as usize;
        Some(section)
    }

    #[inline(always)]
    pub fn get_block_light(&mut self, pos: BlockPos) -> u8 {
        let chunk_x = pos.0.x >> 4;
        let chunk_z = pos.0.z >> 4;

        let Some(idx) = self.get_chunk_index_cached(chunk_x, chunk_z) else {
            return 0;
        };

        let Some(section_y) = self.get_section_y(pos.0.y) else {
            return 0;
        };

        let x = (pos.0.x & 15) as usize;
        let y = (pos.0.y & 15) as usize;
        let z = (pos.0.z & 15) as usize;

        match &self.cache.chunks[idx] {
            Chunk::Level(c) => {
                let light_engine = c.light_engine.lock().unwrap();
                if section_y >= light_engine.block_light.len() {
                    return 0;
                }
                light_engine.block_light[section_y].get(x, y, z)
            }
            Chunk::Proto(c) => {
                if section_y >= c.light.block_light.len() {
                    return 0;
                }
                c.light.block_light[section_y].get(x, y, z)
            }
        }
    }

    #[inline(always)]
    pub fn set_block_light(&mut self, pos: BlockPos, level: u8) {
        let chunk_x = pos.0.x >> 4;
        let chunk_z = pos.0.z >> 4;

        let Some(idx) = self.get_chunk_index_cached(chunk_x, chunk_z) else {
            return;
        };

        let Some(section_y) = self.get_section_y(pos.0.y) else {
            return;
        };

        let x = (pos.0.x & 15) as usize;
        let y = (pos.0.y & 15) as usize;
        let z = (pos.0.z & 15) as usize;

        match &mut self.cache.chunks[idx] {
            Chunk::Level(c) => {
                let mut light_engine = c.light_engine.lock().unwrap();
                if section_y < light_engine.block_light.len() {
                    light_engine.block_light[section_y].set(x, y, z, level);
                    c.dirty.store(true, std::sync::atomic::Ordering::Relaxed);
                }
            }
            Chunk::Proto(c) => {
                if section_y < c.light.block_light.len() {
                    c.light.block_light[section_y].set(x, y, z, level);
                }
            }
        }
    }

    #[inline(always)]
    pub fn get_sky_light(&mut self, pos: BlockPos) -> u8 {
        let chunk_x = pos.0.x >> 4;
        let chunk_z = pos.0.z >> 4;

        let Some(idx) = self.get_chunk_index_cached(chunk_x, chunk_z) else {
            return 0;
        };

        let Some(section_y) = self.get_section_y(pos.0.y) else {
            return 0;
        };

        let x = (pos.0.x & 15) as usize;
        let y = (pos.0.y & 15) as usize;
        let z = (pos.0.z & 15) as usize;

        match &self.cache.chunks[idx] {
            Chunk::Level(c) => {
                let light_engine = c.light_engine.lock().unwrap();
                if section_y >= light_engine.sky_light.len() {
                    return 0;
                }
                light_engine.sky_light[section_y].get(x, y, z)
            }
            Chunk::Proto(c) => {
                if section_y >= c.light.sky_light.len() {
                    return 0;
                }
                c.light.sky_light[section_y].get(x, y, z)
            }
        }
    }

    #[inline(always)]
    pub fn set_sky_light(&mut self, pos: BlockPos, level: u8) {
        let chunk_x = pos.0.x >> 4;
        let chunk_z = pos.0.z >> 4;

        let Some(idx) = self.get_chunk_index_cached(chunk_x, chunk_z) else {
            return;
        };

        let Some(section_y) = self.get_section_y(pos.0.y) else {
            return;
        };

        let x = (pos.0.x & 15) as usize;
        let y = (pos.0.y & 15) as usize;
        let z = (pos.0.z & 15) as usize;

        match &mut self.cache.chunks[idx] {
            Chunk::Level(c) => {
                let mut light_engine = c.light_engine.lock().unwrap();
                if section_y < light_engine.sky_light.len() {
                    light_engine.sky_light[section_y].set(x, y, z, level);
                    c.dirty.store(true, std::sync::atomic::Ordering::Relaxed);
                }
            }
            Chunk::Proto(c) => {
                if section_y < c.light.sky_light.len() {
                    c.light.sky_light[section_y].set(x, y, z, level);
                }
            }
        }
    }

    /// Get block opacity using the underlying cache
    #[inline(always)]
    pub fn get_block_opacity(&mut self, pos: BlockPos) -> u8 {
        let raw = self.cache.get_block_state(&pos.0);
        raw.to_state().opacity
    }
}