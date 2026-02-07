use crate::chunk_system::generation_cache::Cache;
use crate::chunk_system::chunk_state::Chunk;
use crate::generation::height_limit::HeightLimitView;
use crate::generation::proto_chunk::GenerationCache;
use crate::lighting::storage::{get_sky_light, set_sky_light};
use pumpkin_data::BlockDirection;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_util::HeightMap;
use std::collections::{VecDeque, HashSet, HashMap};
use std::sync::atomic::Ordering::Relaxed;

pub struct SkyLightEngine {
    pub(crate) queue: VecDeque<BlockPos>,
    visited: HashSet<BlockPos>,
    decrease_queue: VecDeque<(BlockPos, u8)>,
}

impl SkyLightEngine {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            visited: HashSet::new(),
            decrease_queue: VecDeque::new(),
        }
    }

    /// Clear all internal state to free memory after lighting calculation
    pub fn clear(&mut self) {
        self.queue.clear();
        self.visited.clear();
        self.decrease_queue.clear();
        // Shrink capacity to release memory
        self.queue.shrink_to_fit();
        self.visited.shrink_to_fit();
        self.decrease_queue.shrink_to_fit();
    }

    pub fn convert_light(&mut self, _cache: &mut Cache) {
        // Placeholder or integrated into propagate
    }

    pub fn propagate_light(&mut self, cache: &mut Cache) {
        let center_x = cache.x + (cache.size / 2);
        let center_z = cache.z + (cache.size / 2);

        let min_y = cache.bottom_y() as i32;
        let height = cache.height() as i32;
        let max_y = min_y + height;

        // Reserve queue capacity upfront to avoid repeated reallocations.
        // Up to one quarter of blocks in the column set initially as an estimate.
        self.queue.clear();
        self.visited.clear();
        let estimate = ((16 * 16 * height) / 4).max(1) as usize;
        self.queue.reserve(estimate);
        self.visited.reserve(estimate);

        // Process center chunk + 1-block margin on each side to initialize edge lighting
        // This eliminates the need for expensive boundary seeding with thousands of lock acquisitions
        let start_x = center_x * 16 - 1;
        let start_z = center_z * 16 - 1;
        let end_x = start_x + 18; // 16 + 2 for margins
        let end_z = start_z + 18;

        // Initialize direct sky light for center chunk using heightmap/top-y to avoid
        // scanning the full column. We still set full sky light for air above the
        // top block within this cache region, then scan downward from the top block.
        for z in start_z..end_z {
            for x in start_x..end_x {
                let top_y = cache.get_top_y(&HeightMap::WorldSurface, x, z);

                // Precompute chunk/ local coords once for this column
                let chunk_x = x >> 4;
                let chunk_z = z >> 4;
                let rel_x = chunk_x - cache.x;
                let rel_z = chunk_z - cache.z;
                if rel_x < 0 || rel_x >= cache.size || rel_z < 0 || rel_z >= cache.size {
                    continue;
                }
                let idx = (rel_x * cache.size + rel_z) as usize;
                let local_x = (x & 15) as usize;
                let local_z = (z & 15) as usize;
                let bottom = cache.bottom_y() as i32;

                // If there's air above the top block within our region, set them to full light.
                if top_y < max_y {
                    let mut y = top_y;
                    while y < max_y {
                        let section = ((y - bottom) >> 4) as usize;
                        let section_end_y = ((section as i32 + 1) << 4) + bottom;
                        let end = section_end_y.min(max_y);

                        match &mut cache.chunks[idx] {
                            Chunk::Proto(c) => {
                                if section < c.light.sky_light.len() {
                                    for yy in y..end {
                                        let local_y = (yy & 15) as usize;
                                        let cur = c.light.sky_light[section].get(local_x, local_y, local_z);
                                        if cur != 15 {
                                            c.light.sky_light[section].set(local_x, local_y, local_z, 15);
                                            let pos = BlockPos(Vector3::new(x, yy, z));
                                            if self.visited.insert(pos) {
                                                self.queue.push_back(pos);
                                            }
                                        }
                                    }
                                }
                            }
                            Chunk::Level(c) => {
                                let mut light_engine = c.light_engine.lock().unwrap();
                                let mut changed = false;
                                let section_idx = section as usize;

                                if section_idx < light_engine.sky_light.len() {
                                    for yy in y..end {
                                        let local_y = (yy & 15) as usize;
                                        
                                        // Access the specific section
                                        let light_section = &mut light_engine.sky_light[section_idx];
                                        let cur = light_section.get(local_x, local_y, local_z);
                                        
                                        if cur != 15 {
                                            light_section.set(local_x, local_y, local_z, 15);
                                            changed = true;
                                            
                                            let pos = BlockPos(Vector3::new(x, yy, z));
                                            if self.visited.insert(pos) {
                                                self.queue.push_back(pos);
                                            }
                                        }
                                    }
                                }
                                
                                // If any block light changed, mark the chunk as dirty
                                if changed {
                                    c.dirty.store(true, std::sync::atomic::Ordering::Relaxed);
                                }
                            }
                        }

                        y = end;
                    }
                }

                // Now scan downward from the top block (top_y - 1) down to min_y,
                // attenuating light when encountering opaque blocks.
                let mut light: i32 = 15;
                let start_y = (top_y - 1).min(max_y - 1);
                if start_y >= min_y {
                    let mut y = start_y;
                    'down: while y >= min_y {
                        let section = ((y - bottom) >> 4) as usize;
                        let section_start_y = (section as i32 * 16) + bottom;
                        let start = section_start_y.max(min_y);

                        // Precompute opacities for this section range to avoid borrowing cache
                        // while holding a mutable reference to the chunk.
                        let mut opacities: Vec<u8> = Vec::with_capacity((y - start + 1) as usize);
                        for yy in start..=y {
                            let pos_vec = Vector3::new(x, yy, z);
                            let state = cache.get_block_state(&pos_vec);
                            opacities.push(state.to_state().opacity);
                        }

                        match &mut cache.chunks[idx] {
                            Chunk::Proto(c) => {
                                if section < c.light.sky_light.len() {
                                    // iterate opacities in reverse (from y down to start)
                                    for (i, &opacity) in opacities.iter().enumerate().rev() {
                                        let yy = start + i as i32;
                                        if opacity > 0 {
                                            light = light.saturating_sub(opacity as i32);
                                            if light <= 0 {
                                                break 'down;
                                            }
                                        }
                                        let local_y = (yy & 15) as usize;
                                        let cur = c.light.sky_light[section].get(local_x, local_y, local_z);
                                        if cur != (light as u8) {
                                            c.light.sky_light[section].set(local_x, local_y, local_z, light as u8);
                                            let pos = BlockPos(Vector3::new(x, yy, z));
                                            if self.visited.insert(pos) {
                                                self.queue.push_back(pos);
                                            }
                                        }
                                    }
                                } else {
                                    break 'down;
                                }
                            }
                            Chunk::Level(c) => {
                                let mut light_engine = c.light_engine.lock().unwrap();
                                let mut changed = false;
                                let section_idx = section as usize;

                                if section_idx < light_engine.sky_light.len() {
                                    for (i, &opacity) in opacities.iter().enumerate().rev() {
                                        let yy = start + i as i32;
                                        if opacity > 0 {
                                            light = light.saturating_sub(opacity as i32);
                                            if light <= 0 {
                                                // Break out of the loop
                                                break 'down;
                                            }
                                        }
                                        
                                        let local_y = (yy & 15) as usize;
                                        
                                        // Access the container from the locked engine
                                        let light_section = &mut light_engine.sky_light[section_idx];
                                        let cur = light_section.get(local_x, local_y, local_z);
                                        
                                        if cur != (light as u8) {
                                            light_section.set(local_x, local_y, local_z, light as u8);
                                            changed = true;
                                            
                                            let pos = BlockPos(Vector3::new(x, yy, z));
                                            if self.visited.insert(pos) {
                                                self.queue.push_back(pos);
                                            }
                                        }
                                    }
                                } else {
                                    break 'down;
                                }

                                // Mark dirty if any light levels were updated
                                if changed {
                                    c.dirty.store(true, std::sync::atomic::Ordering::Relaxed);
                                }
                            }
                        }

                        y = start - 1;
                    }
                }
            }
        }

        // Horizontal spread (BFS) with batched updates
        // Propagate light horizontally using flood fill
        let mut pending_updates: HashMap<(i32, i32), Vec<(BlockPos, u8)>> = HashMap::with_capacity(32);
        let mut shadow_cache: HashMap<BlockPos, u8> = HashMap::with_capacity(4096);
        
        while let Some(pos) = self.queue.pop_front() {
            // Read from shadow cache first, then fall back to actual storage
            let level = shadow_cache.get(&pos).copied().unwrap_or_else(|| get_sky_light(cache, pos));
            if level <= 1 {
                continue;
            } // Light level 0 and 1 don't propagate further

            for face in BlockDirection::all() {
                let offset = face.to_offset();
                let neighbor_pos = BlockPos(pos.0 + offset);
                
                // Skip if already visited
                if self.visited.contains(&neighbor_pos) {
                    continue;
                }
                
                // Read from shadow cache first, then fall back to actual storage
                let neighbor_level = shadow_cache.get(&neighbor_pos).copied()
                    .unwrap_or_else(|| get_sky_light(cache, neighbor_pos));
                
                // Query opacity on-demand
                let opacity = cache.get_block_state(&neighbor_pos.0).to_state().opacity.max(1);

                let new_level = level.saturating_sub(opacity);

                // Only update if the new light level is brighter than current
                if new_level > neighbor_level {
                    let chunk_x = neighbor_pos.0.x >> 4;
                    let chunk_z = neighbor_pos.0.z >> 4;
                    pending_updates.entry((chunk_x, chunk_z))
                        .or_default()
                        .push((neighbor_pos, new_level));
                    shadow_cache.insert(neighbor_pos, new_level);
                    
                    if new_level > 1 && self.visited.insert(neighbor_pos) {
                        self.queue.push_back(neighbor_pos);
                    }
                }
            }
            
            // Apply batched updates every 256 blocks to balance memory and lock overhead
            if pending_updates.values().map(|v| v.len()).sum::<usize>() >= 256 {
                self.apply_batched_updates(cache, &mut pending_updates);
            }
        }
        
        // Apply any remaining updates
        if !pending_updates.is_empty() {
            self.apply_batched_updates(cache, &mut pending_updates);
        }
        
        // Process any light decrease operations to handle removed light sources
        self.process_decrease_queue(cache);
    }

    /// Validates lighting and fixes any incorrect light values
    pub fn validate_light(&mut self, cache: &mut Cache) {
        let center_x = cache.x + (cache.size / 2);
        let center_z = cache.z + (cache.size / 2);
        let min_y = cache.bottom_y() as i32;
        let height = cache.height() as i32;
        let max_y = min_y + height;
        
        let start_x = center_x * 16;
        let start_z = center_z * 16;
        let end_x = start_x + 16;
        let end_z = start_z + 16;
        
        // Scan center chunk for invalid lighting
        for y in min_y..max_y {
            for z in start_z..end_z {
                for x in start_x..end_x {
                    let pos = BlockPos(Vector3::new(x, y, z));
                    let current_light = get_sky_light(cache, pos);
                    
                    if current_light == 0 {
                        continue;
                    }
                    
                    // For sky light, check if this position should have sky visibility
                    let mut max_neighbor_light = 0u8;
                    
                    for face in BlockDirection::all() {
                        let offset = face.to_offset();
                        let neighbor_pos = BlockPos(Vector3::new(x, y, z) + offset);
                        let neighbor_light = get_sky_light(cache, neighbor_pos);
                        let neighbor_state = cache.get_block_state(&neighbor_pos.0);
                        let opacity = neighbor_state.to_state().opacity.max(1);
                        
                        let propagated = neighbor_light.saturating_sub(opacity);
                        max_neighbor_light = max_neighbor_light.max(propagated);
                    }
                    
                    // If current light is higher than justified, queue for removal
                    if current_light > max_neighbor_light && current_light < 15 {
                        // Don't remove full sky light (15) as it's from direct sky access
                        self.decrease_queue.push_back((pos, current_light));
                        set_sky_light(cache, pos, 0);
                    }
                }
            }
        }
        
        // Process the decrease queue
        if !self.decrease_queue.is_empty() {
            self.process_decrease_queue(cache);
        }
    }

    /// Process light decrease queue to properly remove light when sources are removed
    fn process_decrease_queue(&mut self, cache: &mut Cache) {
        let mut pending_updates: HashMap<(i32, i32), Vec<(BlockPos, u8)>> = HashMap::new();
        let mut relight_queue: Vec<(BlockPos, u8)> = Vec::new();
        
        while let Some((pos, removed_level)) = self.decrease_queue.pop_front() {
            for face in BlockDirection::all() {
                let offset = face.to_offset();
                let neighbor_pos = BlockPos(pos.0 + offset);
                let neighbor_level = get_sky_light(cache, neighbor_pos);
                
                if neighbor_level == 0 {
                    continue;
                }
                
                let state = cache.get_block_state(&neighbor_pos.0);
                let opacity = state.to_state().opacity.max(1);
                let expected_from_removed = removed_level.saturating_sub(opacity);
                
                // If this neighbor was lit by the removed source, darken it
                if neighbor_level <= expected_from_removed {
                    let chunk_x = neighbor_pos.0.x >> 4;
                    let chunk_z = neighbor_pos.0.z >> 4;
                    pending_updates.entry((chunk_x, chunk_z))
                        .or_default()
                        .push((neighbor_pos, 0));
                    self.decrease_queue.push_back((neighbor_pos, neighbor_level));
                } else {
                    // This neighbor has a brighter source, re-propagate from it
                    relight_queue.push((neighbor_pos, neighbor_level));
                }
            }
        }
        
        // Apply all darkness updates
        if !pending_updates.is_empty() {
            self.apply_batched_updates(cache, &mut pending_updates);
        }
        
        // Re-propagate from remaining light sources
        for (pos, _level) in relight_queue {
            self.queue.push_back(pos);
            self.visited.clear(); // Reset visited for re-propagation
        }
        
        // Re-run propagation if we have sources to relight from
        if !self.queue.is_empty() {
            let mut pending_updates: HashMap<(i32, i32), Vec<(BlockPos, u8)>> = HashMap::new();
            let mut shadow_cache: HashMap<BlockPos, u8> = HashMap::new();
            
            while let Some(pos) = self.queue.pop_front() {
                let level = shadow_cache.get(&pos).copied().unwrap_or_else(|| get_sky_light(cache, pos));
                if level <= 1 {
                    continue;
                }
                
                for face in BlockDirection::all() {
                    let offset = face.to_offset();
                    let neighbor_pos = BlockPos(pos.0 + offset);
                    
                    if self.visited.contains(&neighbor_pos) {
                        continue;
                    }
                    
                    let neighbor_level = shadow_cache.get(&neighbor_pos).copied()
                        .unwrap_or_else(|| get_sky_light(cache, neighbor_pos));
                    let state = cache.get_block_state(&neighbor_pos.0);
                    let opacity = state.to_state().opacity.max(1);
                    let new_level = level.saturating_sub(opacity);
                    
                    if new_level > neighbor_level {
                        let chunk_x = neighbor_pos.0.x >> 4;
                        let chunk_z = neighbor_pos.0.z >> 4;
                        pending_updates.entry((chunk_x, chunk_z))
                            .or_default()
                            .push((neighbor_pos, new_level));
                        shadow_cache.insert(neighbor_pos, new_level);
                        
                        if new_level > 1 && self.visited.insert(neighbor_pos) {
                            self.queue.push_back(neighbor_pos);
                        }
                    }
                }
                
                if pending_updates.values().map(|v| v.len()).sum::<usize>() >= 256 {
                    self.apply_batched_updates(cache, &mut pending_updates);
                }
            }
            
            if !pending_updates.is_empty() {
                self.apply_batched_updates(cache, &mut pending_updates);
            }
        }
    }

    /// Applies batched light updates for a set of chunks
    fn apply_batched_updates(&mut self, cache: &mut Cache, pending_updates: &mut HashMap<(i32, i32), Vec<(BlockPos, u8)>>) {
        let bottom_y = cache.bottom_y() as i32;
        
        for ((chunk_x, chunk_z), updates) in pending_updates.drain() {
            let rel_x = chunk_x - cache.x;
            let rel_z = chunk_z - cache.z;
            if rel_x < 0 || rel_x >= cache.size || rel_z < 0 || rel_z >= cache.size {
                continue;
            }
            let idx = (rel_x * cache.size + rel_z) as usize;
            
            match &mut cache.chunks[idx] {
                Chunk::Level(c) => {
                    let mut light_engine = c.light_engine.lock().unwrap();
                    let mut changed = false;

                    for (pos, level) in updates {
                        let section_y = ((pos.0.y - bottom_y) >> 4) as usize;
                        if section_y < light_engine.sky_light.len() {
                            let x = (pos.0.x & 15) as usize;
                            let y = (pos.0.y & 15) as usize;
                            let z = (pos.0.z & 15) as usize;
                            
                            light_engine.sky_light[section_y].set(x, y, z, level);
                            changed = true;
                        }
                    }
                    
                    if changed {
                        c.dirty.store(true, Relaxed);
                    }
                }
                Chunk::Proto(c) => {
                    for (pos, level) in updates {
                        let section_y = ((pos.0.y - bottom_y) >> 4) as usize;
                        if section_y < c.light.sky_light.len() {
                            let x = (pos.0.x & 15) as usize;
                            let y = (pos.0.y & 15) as usize;
                            let z = (pos.0.z & 15) as usize;
                            c.light.sky_light[section_y].set(x, y, z, level);
                        }
                    }
                }
            }
        }
    }
}

impl Default for SkyLightEngine {
    fn default() -> Self {
        Self::new()
    }
}