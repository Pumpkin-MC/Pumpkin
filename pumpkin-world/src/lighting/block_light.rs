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
    visited: HashSet<BlockPos>,
}

impl BlockLightEngine {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            visited: HashSet::new(),
        }
    }

    pub fn propagate_light(&mut self, cache: &mut Cache) {
        let center_x = cache.x + (cache.size / 2);
        let center_z = cache.z + (cache.size / 2);
        let min_y = cache.bottom_y() as i32;
        let height = cache.height() as i32;
        let max_y = min_y + height;

        let start_x = center_x * 16;
        let start_z = center_z * 16;
        let end_x = start_x + 16;
        let end_z = start_z + 16;

        self.queue.clear();
        self.visited.clear();

        // Initialize light sources in the center chunk
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
        
        // Add edge blocks from neighboring chunks to the queue to ensure proper
        // light propagation across chunk boundaries
        self.seed_boundary_lights(cache, center_x, center_z, min_y, max_y);

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
                let state = cache.get_block_state(&neighbor_pos.0);

                // Uses max(1, opacity) to ensure minimum 1 level reduction per block
                let opacity = state.to_state().opacity.max(1);
                let new_level = level.saturating_sub(opacity);

                // Only update if new light level is brighter
                if new_level > neighbor_level {
                    let chunk_x = neighbor_pos.0.x >> 4;
                    let chunk_z = neighbor_pos.0.z >> 4;
                    pending_updates.entry((chunk_x, chunk_z))
                        .or_insert_with(Vec::new)
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

    /// Seeds the light propagation queue with blocks at chunk boundaries
    /// to ensure light properly propagates across chunk edges
    fn seed_boundary_lights(&mut self, cache: &mut Cache, center_x: i32, center_z: i32, min_y: i32, max_y: i32) {
        let start_x = center_x * 16;
        let start_z = center_z * 16;
        let end_x = start_x + 16;
        let end_z = start_z + 16;
        
        // Check all four edges of the center chunk
        for y in min_y..max_y {
            // West edge (x = start_x - 1)
            for z in start_z..end_z {
                let pos = BlockPos(Vector3::new(start_x - 1, y, z));
                let level = get_block_light(cache, pos);
                if level > 1 && self.visited.insert(pos) {
                    self.queue.push_back(pos);
                }
            }
            
            // East edge (x = end_x)
            for z in start_z..end_z {
                let pos = BlockPos(Vector3::new(end_x, y, z));
                let level = get_block_light(cache, pos);
                if level > 1 && self.visited.insert(pos) {
                    self.queue.push_back(pos);
                }
            }
            
            // North edge (z = start_z - 1)
            for x in start_x..end_x {
                let pos = BlockPos(Vector3::new(x, y, start_z - 1));
                let level = get_block_light(cache, pos);
                if level > 1 && self.visited.insert(pos) {
                    self.queue.push_back(pos);
                }
            }
            
            // South edge (z = end_z)
            for x in start_x..end_x {
                let pos = BlockPos(Vector3::new(x, y, end_z));
                let level = get_block_light(cache, pos);
                if level > 1 && self.visited.insert(pos) {
                    self.queue.push_back(pos);
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
