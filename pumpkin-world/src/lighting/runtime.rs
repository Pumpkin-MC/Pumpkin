use crossbeam::queue::SegQueue;
use std::sync::Arc;
use crate::level::Level;
use pumpkin_util::math::position::BlockPos;
use pumpkin_config::lighting::LightingEngineConfig;
use pumpkin_data::BlockDirection;
use crate::chunk::palette::BlockPalette;
use crate::chunk::io::Dirtiable;

pub struct DynamicLightEngine {
    decrease_block_light_queue: SegQueue<(BlockPos, u8)>,
    increase_block_light_queue: SegQueue<(BlockPos, u8)>,
    decrease_sky_light_queue: SegQueue<(BlockPos, u8)>,
    increase_sky_light_queue: SegQueue<(BlockPos, u8)>,
}

impl DynamicLightEngine {
    pub fn new() -> Self {
        Self {
            decrease_block_light_queue: SegQueue::new(),
            increase_block_light_queue: SegQueue::new(),
            decrease_sky_light_queue: SegQueue::new(),
            increase_sky_light_queue: SegQueue::new(),
        }
    }
}
impl DynamicLightEngine {
    /// Handles all lighting updates triggered by a block change (placement/break).
    /// This updates Block Light, Sky Light, and ensures the source block is valid.
    pub async fn update_lighting_at(&self, level: &Arc<Level>, pos: BlockPos) {
        // 1. Block Light
        self.check_block_light_updates(level, pos).await;
        self.perform_block_light_updates(level).await;

        // 2. Sky Light
        self.check_sky_light_updates(level, pos).await;
        self.perform_sky_light_updates(level).await;

        // 3. Final Safety Check
        // Ensure the source block itself has the correct light level 
        // (e.g. if we just placed a torch, force it to 14 even if propagation was weird)
        // let final_state = level.get_block_state(&pos).await;
        // let final_expected_light = final_state.luminance;
        
        // We unwrap here because if we can't set light on a loaded block, the chunk is broken
        // self.set_block_light_level(level, &pos, final_expected_light)
        //     .await
        //     .expect("Failed to set final light level");
    }

    pub fn queue_block_light_decrease(&self, pos: BlockPos, level: u8) {
        self.decrease_block_light_queue.push((pos, level));
    }

    pub fn queue_block_light_increase(&self, pos: BlockPos, level: u8) {
        self.increase_block_light_queue.push((pos, level));
    }

    pub fn queue_sky_light_decrease(&self, pos: BlockPos, level: u8) {
        self.decrease_sky_light_queue.push((pos, level));
    }

    pub fn queue_sky_light_increase(&self, pos: BlockPos, level: u8) {
        self.increase_sky_light_queue.push((pos, level));
    }

    pub async fn perform_block_light_updates(&self, level: &Arc<Level>) -> i32 {
        let mut updates = 0;

        // Keep processing until both queues are empty
        // Light propagation queues new updates, so we need to process until convergence
        loop {
            let decrease_updates = self.perform_block_light_decrease_updates(level).await;
            let increase_updates = self.perform_block_light_increase_updates(level).await;
            
            updates += decrease_updates + increase_updates;
            
            // Stop when no more updates were processed
            if decrease_updates == 0 && increase_updates == 0 {
                break;
            }
        }

        updates
    }

    async fn perform_block_light_decrease_updates(&self, level: &Arc<Level>) -> i32 {
        let mut updates = 0;

        while let Some((pos, expected_light)) = self.decrease_block_light_queue.pop() {
            self.propagate_block_light_decrease(level, &pos, expected_light)
                .await;
            updates += 1;
        }

        updates
    }

    async fn perform_block_light_increase_updates(&self, level: &Arc<Level>) -> i32 {
        let mut updates = 0;

        while let Some((pos, expected_light)) = self.increase_block_light_queue.pop() {
            self.propagate_block_light_increase(level, &pos, expected_light)
                .await;
            updates += 1;
        }

        updates
    }

    async fn propagate_block_light_increase(&self, level: &Arc<Level>, pos: &BlockPos, light_level: u8) {
        for dir in BlockDirection::all() {
            let neighbor_pos = pos.offset(dir.to_offset());

            if let Some(neighbor_light) = self.get_block_light_level(level, &neighbor_pos).await {
                let neighbor_state = level.get_block_state(&neighbor_pos).await.to_state();
                let opacity = neighbor_state.opacity.max(1);
                let new_light = light_level.saturating_sub(opacity);

                // Only propagate if new light is brighter than current light
                if new_light > neighbor_light {
                    self.set_block_light_level(level, &neighbor_pos, new_light)
                        .await
                        .unwrap();

                    if new_light > 1 {
                        self.queue_block_light_increase(neighbor_pos, new_light);
                    }
                }
            }
        }
    }

    async fn propagate_block_light_decrease(
        &self, level: &Arc<Level>,
        pos: &BlockPos,
        removed_light_level: u8,
    ) {
        // Check what the current light level actually is at this position
        let current_level = self.get_block_light_level(level, pos).await.unwrap_or(0);
        
        // Only propagate decrease if this position hasn't already been reset to 0
        // This prevents positions that were intentionally set to 0 from propagating light
        if current_level == 0 && removed_light_level > 0 {
            // This position was already darkened, so we propagate the darkness to neighbors
            for dir in BlockDirection::all() {
                let neighbor_pos = pos.offset(dir.to_offset());

                if let Some(neighbor_light) = self.get_block_light_level(level, &neighbor_pos).await {
                    if neighbor_light == 0 {
                        continue; // Skip if already 0
                    }

                    let neighbor_state = level.get_block_state(&neighbor_pos).await.to_state();
                    let opacity = neighbor_state.opacity.max(1);

                    let expected_from_removed_source = removed_light_level.saturating_sub(opacity);

                    if neighbor_light <= expected_from_removed_source {
                        let neighbor_luminance = neighbor_state.luminance;

                        if neighbor_luminance == 0 {
                            // No self-emission, darken it completely and continue propagation
                            self.set_block_light_level(level, &neighbor_pos, 0).await.unwrap();
                            self.queue_block_light_decrease(neighbor_pos, neighbor_light);
                        } else {
                            // Has self-emission, set to its own light and re-propagate from it
                            self.set_block_light_level(level, &neighbor_pos, neighbor_luminance)
                                .await
                                .unwrap();
                            self.queue_block_light_increase(neighbor_pos, neighbor_luminance);
                        }
                    } else {
                        // This neighbor has brighter light from another source, re-propagate from it
                        self.queue_block_light_increase(neighbor_pos, neighbor_light);
                    }
                }
            }
        }
    }

    pub async fn check_block_light_updates(&self, level: &Arc<Level>, pos: BlockPos) {
        match level.lighting_config {
            LightingEngineConfig::Full => {
                self.set_block_light_level(level, &pos, 15).await.unwrap();
                return;
            }
            LightingEngineConfig::Dark => {
                self.set_block_light_level(level, &pos, 0).await.unwrap();
                return;
            }
            LightingEngineConfig::Default => {}
        }

        let current_light = self.get_block_light_level(level, &pos).await.unwrap_or(0);
        let block_state = level.get_block_state(&pos).await.to_state();
        let expected_light = block_state.luminance;

        // Handle light decrease (removing light source or placing opaque block)
        if expected_light < current_light {
            // Set to expected value immediately, then queue decrease to darken neighbors
            self.set_block_light_level(level, &pos, expected_light).await.unwrap();
            self.queue_block_light_decrease(pos, current_light);
        } else if expected_light > current_light {
            // Handle light increase (placing light source)
            self.set_block_light_level(level, &pos, expected_light)
                .await
                .unwrap();
            self.queue_block_light_increase(pos, expected_light);
        }

        // Only check neighbors if we didn't trigger a decrease
        // Decrease propagation handles re-validating neighbors
        if expected_light >= current_light {
            self.check_neighbors_light_updates(level, pos, expected_light).await;
        }
    }

    pub async fn check_neighbors_light_updates(&self, level: &Arc<Level>, pos: BlockPos, current_light: u8) {
        for dir in BlockDirection::all() {
            let neighbor_pos = pos.offset(dir.to_offset());
            if let Some(neighbor_light) = self.get_block_light_level(level, &neighbor_pos).await
                && neighbor_light > current_light + 1
            {
                self.queue_block_light_increase(neighbor_pos, neighbor_light);
            }
        }
    }

    pub async fn perform_sky_light_updates(&self, level: &Arc<Level>) -> i32 {
        let mut updates = 0;
        loop {
            let decrease_updates = self.perform_sky_light_decrease_updates(level).await;
            let increase_updates = self.perform_sky_light_increase_updates(level).await;
            
            updates += decrease_updates + increase_updates;
            
            if decrease_updates == 0 && increase_updates == 0 {
                break;
            }
        }
        updates
    }

    async fn perform_sky_light_decrease_updates(&self, level: &Arc<Level>) -> i32 {
        let mut updates = 0;
        while let Some((pos, expected_light)) = self.decrease_sky_light_queue.pop() {
            self.propagate_sky_light_decrease(level, &pos, expected_light).await;
            updates += 1;
        }
        updates
    }

    async fn perform_sky_light_increase_updates(&self, level: &Arc<Level>) -> i32 {
        let mut updates = 0;
        while let Some((pos, expected_light)) = self.increase_sky_light_queue.pop() {
            self.propagate_sky_light_increase(level, &pos, expected_light).await;
            updates += 1;
        }
        updates
    }

    async fn propagate_sky_light_increase(&self, level: &Arc<Level>, pos: &BlockPos, light_level: u8) {
        for dir in BlockDirection::all() {
            let neighbor_pos = pos.offset(dir.to_offset());
            if let Some(neighbor_light) = self.get_sky_light_level(level, &neighbor_pos).await {
                // If propagating DOWN from 15, result is 15 (if transparency allows)
                let potential_light = if light_level == 15 && dir == BlockDirection::Down {
                    15
                } else {
                    light_level.saturating_sub(1)
                };

                let neighbor_state = level.get_block_state(&neighbor_pos).await.to_state();
                let opacity = neighbor_state.opacity;
                let new_light = potential_light.saturating_sub(opacity);

                // Only propagate if new light is brighter than current light
                if new_light > neighbor_light {
                    self.set_sky_light_level(level, &neighbor_pos, new_light).await.unwrap();
                    self.queue_sky_light_increase(neighbor_pos, new_light);
                }
            }
        }
    }

    async fn propagate_sky_light_decrease(&self, level: &Arc<Level>, pos: &BlockPos, removed_light: u8) {
         let current_light = self.get_sky_light_level(level, pos).await.unwrap_or(0);

         // If we are brighter now (maybe re-illuminated), don't propagate darkness
         if current_light >= removed_light { return; }

         for dir in BlockDirection::all() {
             let neighbor_pos = pos.offset(dir.to_offset());
             if let Some(neighbor_light) = self.get_sky_light_level(level, &neighbor_pos).await {
                 if neighbor_light == 0 { continue; }

                 // Check if neighbor was likely lit by us
                 // Inverse of increase logic
                 let potential_from_us = if removed_light == 15 && dir == BlockDirection::Down {
                     15
                 } else {
                     removed_light.saturating_sub(1)
                 };
                 let neighbor_state = level.get_block_state(&neighbor_pos).await.to_state();
                 let opacity = neighbor_state.opacity;
                 let expected = potential_from_us.saturating_sub(opacity);

                 if neighbor_light == expected || neighbor_light < removed_light {
                     // Darken and propagate
                     self.set_sky_light_level(level, &neighbor_pos, 0).await.unwrap();
                     self.queue_sky_light_decrease(neighbor_pos, neighbor_light);
                 } else if neighbor_light >= removed_light {
                     // Neighbor is a source/brighter, re-propagate
                     self.queue_sky_light_increase(neighbor_pos, neighbor_light);
                 }
             }
         }
    }

    pub async fn check_sky_light_updates(&self, level: &Arc<Level>, pos: BlockPos) {
        match level.lighting_config {
            LightingEngineConfig::Full => {
                self.set_sky_light_level(level, &pos, 15).await.unwrap();
                return;
            }
            LightingEngineConfig::Dark => {
                self.set_sky_light_level(level, &pos, 0).await.unwrap();
                return;
            }
            LightingEngineConfig::Default => {}
        }
        
        let current_light = self.get_sky_light_level(level, &pos).await.unwrap_or(0);
        let block_state = level.get_block_state(&pos).await.to_state();
        
        let mut best_light = 0;
        for dir in BlockDirection::all() {
            let neighbor_pos = pos.offset(dir.to_offset());
            if let Some(n_light) = self.get_sky_light_level(level, &neighbor_pos).await {
                // Determine potential light from this neighbor
                let pot = if n_light == 15 && dir == BlockDirection::Up {
                    15
                } else {
                    n_light.saturating_sub(1)
                };
                if pot > best_light { best_light = pot; }
            }
        }

        let opacity = block_state.opacity;
        // Apply opacity decay
        let expected_light = best_light.saturating_sub(opacity);

        if expected_light < current_light {
            self.set_sky_light_level(level, &pos, expected_light).await.unwrap();
            self.queue_sky_light_decrease(pos, current_light);
        } else if expected_light > current_light {
             self.set_sky_light_level(level, &pos, expected_light).await.unwrap();
             self.queue_sky_light_increase(pos, expected_light);
        }

        // Only check/notify neighbors if we didn't just cause a massive drop 
        // (decrease queue handles notifying neighbors to drop their light)
        if expected_light >= current_light {
             self.check_neighbors_sky_light_updates(level, pos, expected_light).await;
        }
    }

    pub async fn check_neighbors_sky_light_updates(&self, level: &Arc<Level>, pos: BlockPos, current_light: u8) {
        // Kickstart neighbors
        for dir in BlockDirection::all() {
            let neighbor_pos = pos.offset(dir.to_offset());
            if self.get_sky_light_level(level, &neighbor_pos).await.is_some() {
                // Simply queueing self will force re-propagation to neighbors
                self.queue_sky_light_increase(pos, current_light);
                break;
            }
        }
    }

    pub async fn get_block_light_level(&self, level: &Arc<Level>, position: &BlockPos) -> Option<u8> {
        let (chunk_coordinate, relative) = position.chunk_and_chunk_relative_position();
        let chunk = level.get_chunk(chunk_coordinate).await;

        let section_index = (relative.y - chunk.section.min_y) as usize / BlockPalette::SIZE;
        // Bounds check for section index (lock the light engine)
        let light_engine = chunk.light_engine.lock().unwrap();
        if section_index >= light_engine.block_light.len() {
            return None;
        }
        Some(light_engine.block_light[section_index].get(
            relative.x as usize,
            (relative.y - chunk.section.min_y) as usize % BlockPalette::SIZE,
            relative.z as usize,
        ))
    }

    #[expect(clippy::unused_async)]
    pub async fn set_block_light_level(
        &self,
        level: &Arc<Level>,
        position: &BlockPos,
        light_level: u8,
    ) -> Result<(), String> {
        let (chunk_coordinate, relative) = position.chunk_and_chunk_relative_position();
        let chunk = level.get_chunk(chunk_coordinate).await;
        let section_index = (relative.y - chunk.section.min_y) as usize / BlockPalette::SIZE;
        // Bounds check for section index
        let mut light_engine = chunk.light_engine.lock().unwrap();
        if section_index >= light_engine.block_light.len() {
            return Err("Invalid section index".to_string());
        }
        let relative_y = (relative.y - chunk.section.min_y) as usize % BlockPalette::SIZE;
        light_engine.block_light[section_index].set(
            relative.x as usize,
            relative_y,
            relative.z as usize,
            light_level,
        );
        // Mark chunk as dirty so lighting changes are saved to disk
        if !chunk.is_dirty() { 
            chunk.mark_dirty(true);
        }
        Ok(())
    }

    pub async fn get_sky_light_level(&self, level: &Arc<Level>, position: &BlockPos) -> Option<u8> {
        let (chunk_coordinate, relative) = position.chunk_and_chunk_relative_position();
        let chunk = level.get_chunk(chunk_coordinate).await;
        let section_index = (relative.y - chunk.section.min_y) as usize / BlockPalette::SIZE;
        // Bounds check for section index (lock the light engine)
        let light_engine = chunk.light_engine.lock().unwrap();
        if section_index >= light_engine.sky_light.len() {
            return None;
        }
        Some(light_engine.sky_light[section_index].get(
            relative.x as usize,
            (relative.y - chunk.section.min_y) as usize % BlockPalette::SIZE,
            relative.z as usize,
        ))
    }

    pub async fn set_sky_light_level(
        &self,
        level: &Arc<Level>,
        position: &BlockPos,
        light_level: u8,
    ) -> Result<(), String> {
        let (chunk_coordinate, relative) = position.chunk_and_chunk_relative_position();
        let chunk = level.get_chunk(chunk_coordinate).await;
        let section_index = (relative.y - chunk.section.min_y) as usize / BlockPalette::SIZE;
        // Bounds check for section index
        let mut light_engine = chunk.light_engine.lock().unwrap();
        if section_index >= light_engine.sky_light.len() {
            return Err("Invalid section index".to_string());
        }
        let relative_y = (relative.y - chunk.section.min_y) as usize % BlockPalette::SIZE;
        light_engine.sky_light[section_index].set(
            relative.x as usize,
            relative_y,
            relative.z as usize,
            light_level,
        );
        // Mark chunk as dirty so lighting changes are saved to disk
        if !chunk.is_dirty() { 
            chunk.mark_dirty(true);
        }
        Ok(())
    }
}