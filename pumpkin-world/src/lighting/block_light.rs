use crate::chunk_system::generation_cache::Cache;
use crate::chunk_system::chunk_state::Chunk;
use crate::generation::height_limit::HeightLimitView;
use crate::generation::proto_chunk::GenerationCache;
use crate::lighting::storage::{get_block_light, set_block_light};
use pumpkin_data::BlockDirection;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use std::collections::{VecDeque, HashSet, HashMap};

pub struct BlockLightEngine {
    pub(crate) queue: VecDeque<BlockPos>,
    pub(super) visited: HashSet<BlockPos>,
    pub(super) decrease_queue: VecDeque<(BlockPos, u8)>,
}

impl BlockLightEngine {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            visited: HashSet::new(),
            decrease_queue: VecDeque::new(),
        }
    }

    pub fn propagate_light(&mut self, cache: &mut Cache) {
        let center_x = cache.x + (cache.size / 2);
        let center_z = cache.z + (cache.size / 2);
        let min_y = cache.bottom_y() as i32;
        let height = cache.height() as i32;
        let max_y = min_y + height;

        // Process center chunk + 1-block margin on each side to initialize edge lighting
        let start_x = center_x * 16 - 1;
        let start_z = center_z * 16 - 1;
        let end_x = start_x + 18;
        let end_z = start_z + 18;

        self.queue.clear();
        self.visited.clear();

        // Initialize light sources in the expanded region (center + margins)
        for y in min_y..max_y {
            for z in start_z..end_z {
                for x in start_x..end_x {
                    let pos_vec = Vector3::new(x, y, z);
                    let state = cache.get_block_state(&pos_vec);
                    let emission = state.to_state().luminance;
                    if emission > 0 {
                        let pos = BlockPos(pos_vec);
                        set_block_light(cache, pos, emission);
                        if self.visited.insert(pos) {
                            self.queue.push_back(pos);
                        }
                    }
                }
            }
        }
        
        // Pre-cache block opacity for the entire working region
        let mut opacity_cache: HashMap<BlockPos, u8> = HashMap::new();
        let cache_margin = 2;
        for y in min_y..max_y {
            for z in (start_z - cache_margin)..(end_z + cache_margin) {
                for x in (start_x - cache_margin)..(end_x + cache_margin) {
                    let pos = BlockPos(Vector3::new(x, y, z));
                    let state = cache.get_block_state(&Vector3::new(x, y, z));
                    let opacity = state.to_state().opacity;
                    if opacity > 0 {
                        opacity_cache.insert(pos, opacity);
                    }
                }
            }
        }

        // BFS propagation with batched updates
        let mut pending_updates: HashMap<(i32, i32), Vec<(BlockPos, u8)>> = HashMap::new();
        let mut shadow_cache: HashMap<BlockPos, u8> = HashMap::new();
        
        while let Some(pos) = self.queue.pop_front() {
            // Read from shadow cache first, then fall back to actual storage
            let level = shadow_cache.get(&pos).copied().unwrap_or_else(|| get_block_light(cache, pos));
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
                    .unwrap_or_else(|| get_block_light(cache, neighbor_pos));

                // Get opacity from cache, or compute if not cached
                let opacity = opacity_cache.get(&neighbor_pos).copied()
                    .unwrap_or_else(|| cache.get_block_state(&neighbor_pos.0).to_state().opacity)
                    .max(1);

                let new_level = level.saturating_sub(opacity);

                // Only update if new light level is brighter
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
        
        // Process any light decrease operations
        self.process_decrease_queue(cache);
    }

    /// Validates lighting and fixes any incorrect light values
    /// This scans for blocks that have light but shouldn't based on their neighbors ("ghost" lights or artifacts)
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
                    let current_light = get_block_light(cache, pos);
                    
                    if current_light == 0 {
                        continue; // No light to validate
                    }
                    
                    let state = cache.get_block_state(&Vector3::new(x, y, z));
                    let self_emission = state.to_state().luminance;
                    
                    // Check if this block's light is justified
                    let mut max_neighbor_light = self_emission;
                    
                    for face in BlockDirection::all() {
                        let offset = face.to_offset();
                        let neighbor_pos = BlockPos(Vector3::new(x, y, z) + offset);
                        let neighbor_light = get_block_light(cache, neighbor_pos);
                        let neighbor_state = cache.get_block_state(&neighbor_pos.0);
                        let opacity = neighbor_state.to_state().opacity.max(1);
                        
                        // What light level could propagate from this neighbor?
                        let propagated = neighbor_light.saturating_sub(opacity);
                        max_neighbor_light = max_neighbor_light.max(propagated);
                    }
                    
                    // If current light is higher than what's justified, it's a ghost light
                    if current_light > max_neighbor_light {
                        self.decrease_queue.push_back((pos, current_light));
                        set_block_light(cache, pos, 0);
                    }
                }
            }
        }
        
        // Process the decrease queue to clean up ghost lights
        if !self.decrease_queue.is_empty() {
            self.process_decrease_queue(cache);
        }
    }

    /// Process light decrease queue to properly remove light when sources are removed.
    pub(in crate::lighting) fn process_decrease_queue(&mut self, cache: &mut Cache) {
        let mut pending_updates: HashMap<(i32, i32), Vec<(BlockPos, u8)>> = HashMap::new();
        let mut relight_queue: Vec<(BlockPos, u8)> = Vec::new();
        
        while let Some((pos, removed_level)) = self.decrease_queue.pop_front() {
            for face in BlockDirection::all() {
                let offset = face.to_offset();
                let neighbor_pos = BlockPos(pos.0 + offset);
                let neighbor_level = get_block_light(cache, neighbor_pos);
                
                if neighbor_level == 0 {
                    continue;
                }
                
                let state = cache.get_block_state(&neighbor_pos.0);
                let opacity = state.to_state().opacity.max(1);
                let expected_from_removed = removed_level.saturating_sub(opacity);
                
                // If this neighbor was lit by the removed source, darken it
                if neighbor_level <= expected_from_removed {
                    let neighbor_luminance = state.to_state().luminance;
                    
                    let chunk_x = neighbor_pos.0.x >> 4;
                    let chunk_z = neighbor_pos.0.z >> 4;
                    
                    if neighbor_luminance == 0 {
                        // No self-emission, darken it
                        pending_updates.entry((chunk_x, chunk_z))
                            .or_default()
                            .push((neighbor_pos, 0));
                        self.decrease_queue.push_back((neighbor_pos, neighbor_level));
                    } else {
                        // Has self-emission, set to its own light level
                        pending_updates.entry((chunk_x, chunk_z))
                            .or_default()
                            .push((neighbor_pos, neighbor_luminance));
                        relight_queue.push((neighbor_pos, neighbor_luminance));
                    }
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
            self.visited.clear();
        }
        
        // Re-run propagation if we have sources to relight from
        if !self.queue.is_empty() {
            let mut pending_updates: HashMap<(i32, i32), Vec<(BlockPos, u8)>> = HashMap::new();
            let mut shadow_cache: HashMap<BlockPos, u8> = HashMap::new();
            
            while let Some(pos) = self.queue.pop_front() {
                let level = shadow_cache.get(&pos).copied().unwrap_or_else(|| get_block_light(cache, pos));
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
                        .unwrap_or_else(|| get_block_light(cache, neighbor_pos));
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
                    let mut write = c.blocking_write();
                    for (pos, level) in updates {
                        let section_y = ((pos.0.y - bottom_y) >> 4) as usize;
                        if section_y < write.light_engine.block_light.len() {
                            let x = (pos.0.x & 15) as usize;
                            let y = (pos.0.y & 15) as usize;
                            let z = (pos.0.z & 15) as usize;
                            write.light_engine.block_light[section_y].set(x, y, z, level);
                            write.dirty = true;
                        }
                    }
                }
                Chunk::Proto(c) => {
                    for (pos, level) in updates {
                        let section_y = ((pos.0.y - bottom_y) >> 4) as usize;
                        if section_y < c.light.block_light.len() {
                            let x = (pos.0.x & 15) as usize;
                            let y = (pos.0.y & 15) as usize;
                            let z = (pos.0.z & 15) as usize;
                            c.light.block_light[section_y].set(x, y, z, level);
                        }
                    }
                }
            }
        }
    }
}

impl Default for BlockLightEngine {
    fn default() -> Self {
        Self::new()
    }
}
