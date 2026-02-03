use crate::chunk_system::generation_cache::Cache;
use crate::generation::height_limit::HeightLimitView;
use crate::generation::proto_chunk::GenerationCache;
use crate::lighting::storage::{get_block_light, set_block_light};
use pumpkin_data::BlockDirection;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use std::collections::VecDeque;

pub struct BlockLightEngine {
    pub(crate) queue: VecDeque<BlockPos>,
}

impl BlockLightEngine {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
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
                        self.queue.push_back(pos);
                    }
                }
            }
        }
        
        // Add edge blocks from neighboring chunks to the queue to ensure proper
        // light propagation across chunk boundaries
        self.seed_boundary_lights(cache, center_x, center_z, min_y, max_y);

        while let Some(pos) = self.queue.pop_front() {
            let level = get_block_light(cache, pos);
            if level <= 1 {
                continue;
            } // Light level 0 and 1 don't propagate further

            for face in BlockDirection::all() {
                let offset = face.to_offset();
                let neighbor_pos = BlockPos(pos.0 + offset);

                let neighbor_level = get_block_light(cache, neighbor_pos);
                let state = cache.get_block_state(&neighbor_pos.0);

                // Uses max(1, opacity) to ensure minimum 1 level reduction per block
                let opacity = state.to_state().opacity.max(1);
                let new_level = level.saturating_sub(opacity);

                // Only update if new light level is brighter
                if new_level > neighbor_level {
                    set_block_light(cache, neighbor_pos, new_level);
                    if new_level > 1 {
                        self.queue.push_back(neighbor_pos);
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
                if level > 1 {
                    self.queue.push_back(pos);
                }
            }
            
            // East edge (x = end_x)
            for z in start_z..end_z {
                let pos = BlockPos(Vector3::new(end_x, y, z));
                let level = get_block_light(cache, pos);
                if level > 1 {
                    self.queue.push_back(pos);
                }
            }
            
            // North edge (z = start_z - 1)
            for x in start_x..end_x {
                let pos = BlockPos(Vector3::new(x, y, start_z - 1));
                let level = get_block_light(cache, pos);
                if level > 1 {
                    self.queue.push_back(pos);
                }
            }
            
            // South edge (z = end_z)
            for x in start_x..end_x {
                let pos = BlockPos(Vector3::new(x, y, end_z));
                let level = get_block_light(cache, pos);
                if level > 1 {
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
