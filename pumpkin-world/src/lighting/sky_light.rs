use crate::chunk_system::generation_cache::Cache;
use crate::lighting::storage::{get_sky_light, set_sky_light};
use crate::generation::height_limit::HeightLimitView;
use crate::generation::proto_chunk::GenerationCache;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_data::BlockDirection;
use std::collections::VecDeque;

pub struct SkyLightEngine {
    pub(crate) queue: VecDeque<BlockPos>,
}

impl SkyLightEngine {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
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

        let start_x = center_x * 16;
        let start_z = center_z * 16;
        let end_x = start_x + 16;
        let end_z = start_z + 16;

        // 1. Initialize Direct Sky Light (Top-Down)
        // Sky light starts at 15 at the top and only decreases when passing through opaque blocks
        for z in start_z..end_z {
            for x in start_x..end_x {
                let mut light: i32 = 15;
                
                // Iterate top-down through the column
                for y in (min_y..max_y).rev() {
                     let pos_vec = Vector3::new(x, y, z);
                     let state = cache.get_block_state(&pos_vec);
                     let opacity = state.to_state().opacity;
                     
                     // Sky light passes through transparent blocks (opacity=0) without attenuation
                     // Only opaque blocks reduce the light level
                     if opacity > 0 {
                         light = light.saturating_sub(opacity as i32);
                         if light <= 0 {
                             break; // No more light propagates below
                         }
                     }
                     
                     // Set the light value and add to queue for horizontal propagation
                     set_sky_light(cache, BlockPos(pos_vec), light as u8);
                     self.queue.push_back(BlockPos(pos_vec));
                }
            }
        }
        
        // 2. Horizontal Spread (BFS)
        // Propagate light horizontally using flood fill
        while let Some(pos) = self.queue.pop_front() {
             let level = get_sky_light(cache, pos);
             if level <= 1 { continue; } // Light level 0 and 1 don't propagate further
             
             for face in BlockDirection::all() {
                  let offset = face.to_offset();
                  let neighbor_pos = BlockPos(pos.0 + offset);
                  let neighbor_level = get_sky_light(cache, neighbor_pos);
                  let state = cache.get_block_state(&neighbor_pos.0);
                  
                  // Calculate light reduction based on block opacity
                  // Vanilla uses max(1, opacity) to ensure at least 1 level reduction per block
                  let opacity = state.to_state().opacity.max(1);
                  let new_level = level.saturating_sub(opacity);
                  
                  // Only update if the new light level is brighter than current
                  if new_level > neighbor_level {
                       set_sky_light(cache, neighbor_pos, new_level);
                       if new_level > 1 {
                           self.queue.push_back(neighbor_pos);
                       }
                  }
             }
        }
    }
}
