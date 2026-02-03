use crate::chunk_system::generation_cache::Cache;
use crate::lighting::storage::{get_block_light, set_block_light};
use crate::generation::height_limit::HeightLimitView;
use crate::generation::proto_chunk::GenerationCache;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_data::BlockDirection;
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
        
        while let Some(pos) = self.queue.pop_front() {
             let level = get_block_light(cache, pos);
             if level == 0 { continue; }
             
             for face in BlockDirection::all() {
                  let offset = face.to_offset();
                  let neighbor_pos = BlockPos(pos.0 + offset);

                  let neighbor_level = get_block_light(cache, neighbor_pos);
                  let state = cache.get_block_state(&neighbor_pos.0);
                  
                  let opacity = state.to_state().opacity.max(1);
                  if (level as i16 - opacity as i16) > neighbor_level as i16 {
                       let new_level = (level as i16 - opacity as i16) as u8;
                       set_block_light(cache, neighbor_pos, new_level);
                       self.queue.push_back(neighbor_pos);
                  }
             }
        }
    }
}
