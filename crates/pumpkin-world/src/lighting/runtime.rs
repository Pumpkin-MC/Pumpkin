use crate::chunk::io::Dirtiable;
use crate::chunk::palette::BlockPalette;
use crate::level::Level;
use crossbeam::queue::SegQueue;
use pumpkin_config::lighting::LightingEngineConfig;
use pumpkin_data::BlockDirection;
use pumpkin_util::math::{position::BlockPos, vector2::Vector2};
use std::collections::HashSet;
use std::sync::Arc;

pub struct DynamicLightEngine {
    block_decrease: SegQueue<(BlockPos, u8)>,
    block_increase: SegQueue<(BlockPos, u8)>,
    sky_decrease: SegQueue<(BlockPos, u8)>,
    sky_increase: SegQueue<(BlockPos, u8)>,
    dirty_chunks: SegQueue<Vector2<i32>>,
}

impl DynamicLightEngine {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            block_decrease: SegQueue::new(),
            block_increase: SegQueue::new(),
            sky_decrease: SegQueue::new(),
            sky_increase: SegQueue::new(),
            dirty_chunks: SegQueue::new(),
        }
    }
}
impl Default for DynamicLightEngine {
    fn default() -> Self {
        Self::new()
    }
}
/// Sky light attenuation for one BFS step, matching vanilla
/// `SkyLightEngine`/`LightEngine.getOpacity` (`fromLevel - max(1, dampening)`,
/// LightEngine.java:77-79, used at SkyLightEngine.java:152/188). The direct-sun
/// column (15 propagating straight down through a non-occluding block) is a
/// separate vanilla mechanic (`ChunkSkyLightSources`/`isSource`) that does not
/// decay at all, kept here as the `down` special case.
fn sky_attenuation(from_level: u8, opacity: u8, down: bool) -> u8 {
    if from_level == 15 && down && opacity == 0 {
        15
    } else {
        from_level.saturating_sub(opacity.max(1))
    }
}

impl DynamicLightEngine {
    /// Checks if there is an open sky above the given position (no opaque blocks blocking sky light).
    fn has_open_sky_above(level: &Arc<Level>, pos: &BlockPos) -> bool {
        let dimension = level.world_gen.dimension();
        let max_y = dimension.min_y + dimension.height - 1;
        let mut current_pos = *pos;

        // Scan upward until we hit sky or an opaque block
        while current_pos.0.y < max_y {
            current_pos.0.y += 1;

            let state = level.get_block_state(&current_pos).to_state();
            if state.opacity > 0 {
                return false; // Hit an opaque block before reaching sky
            }
        }

        true // Reached sky without hitting opaque blocks
    }

    fn has_skylight(level: &Level) -> bool {
        level.world_gen.dimension().has_skylight
    }

    fn mark_chunk_dirty(&self, pos: &BlockPos) {
        let (chunk, _) = pos.chunk_and_chunk_relative_position();
        self.dirty_chunks.push(chunk);
    }

    /// Returns and clears the chunks whose light arrays changed since the last call.
    pub fn take_dirty_chunks(&self) -> Vec<Vector2<i32>> {
        let mut chunks = HashSet::new();
        while let Some(chunk) = self.dirty_chunks.pop() {
            chunks.insert(chunk);
        }
        chunks.into_iter().collect()
    }

    /// Handles all lighting updates triggered by a block change (placement/break).
    /// This updates Block Light, Sky Light, and ensures the source block is valid.
    pub fn update_lighting_at(&self, level: &Arc<Level>, pos: BlockPos) {
        // Block Light
        self.check_block_light_updates(level, pos);
        self.perform_block_light_updates(level);

        // Sky Light
        self.check_sky_light_updates(level, pos);
        self.perform_sky_light_updates(level);
    }

    pub fn queue_block_light_decrease(&self, pos: BlockPos, level: u8) {
        self.block_decrease.push((pos, level));
    }

    pub fn queue_block_light_increase(&self, pos: BlockPos, level: u8) {
        self.block_increase.push((pos, level));
    }

    pub fn queue_sky_light_decrease(&self, pos: BlockPos, level: u8) {
        self.sky_decrease.push((pos, level));
    }

    pub fn queue_sky_light_increase(&self, pos: BlockPos, level: u8) {
        self.sky_increase.push((pos, level));
    }

    pub fn perform_block_light_updates(&self, level: &Arc<Level>) -> i32 {
        let mut updates = 0;

        // Keep processing until both queues are empty
        // Light propagation queues new updates, so we need to process until convergence
        loop {
            let decrease_updates = self.perform_block_light_decrease_updates(level);
            let increase_updates = self.perform_block_light_increase_updates(level);

            updates += decrease_updates + increase_updates;

            // Stop when no more updates were processed
            if decrease_updates == 0 && increase_updates == 0 {
                break;
            }
        }

        updates
    }

    fn perform_block_light_decrease_updates(&self, level: &Arc<Level>) -> i32 {
        let mut updates = 0;

        while let Some((pos, expected_light)) = self.block_decrease.pop() {
            self.propagate_block_light_decrease(level, &pos, expected_light);
            updates += 1;
        }

        updates
    }

    fn perform_block_light_increase_updates(&self, level: &Arc<Level>) -> i32 {
        let mut updates = 0;

        while let Some((pos, expected_light)) = self.block_increase.pop() {
            self.propagate_block_light_increase(level, &pos, expected_light);
            updates += 1;
        }

        updates
    }

    fn propagate_block_light_increase(&self, level: &Arc<Level>, pos: &BlockPos, light_level: u8) {
        for dir in BlockDirection::all() {
            let neighbor_pos = pos.offset(dir.to_offset());

            if let Some(neighbor_light) = self.get_block_light_level(level, &neighbor_pos) {
                let neighbor_state = level.get_block_state(&neighbor_pos).to_state();
                let opacity = neighbor_state.opacity.max(1);
                let new_light = light_level.saturating_sub(opacity);

                // Only propagate if new light is brighter than current light
                if new_light > neighbor_light
                    && self
                        .set_block_light_level(level, &neighbor_pos, new_light)
                        .is_ok()
                    && new_light > 1
                {
                    self.queue_block_light_increase(neighbor_pos, new_light);
                }
            }
        }
    }

    fn propagate_block_light_decrease(
        &self,
        level: &Arc<Level>,
        pos: &BlockPos,
        removed_light_level: u8,
    ) {
        if removed_light_level > 0 {
            // The source may still emit at a lower level; vanilla propagates the old level's
            // decrease first and then re-adds the block's remaining emission.
            for dir in BlockDirection::all() {
                let neighbor_pos = pos.offset(dir.to_offset());

                if let Some(neighbor_light) = self.get_block_light_level(level, &neighbor_pos) {
                    if neighbor_light == 0 {
                        continue; // Skip if already 0
                    }

                    let neighbor_state = level.get_block_state(&neighbor_pos).to_state();
                    let opacity = neighbor_state.opacity.max(1);

                    let expected_from_removed_source = removed_light_level.saturating_sub(opacity);

                    if neighbor_light <= expected_from_removed_source {
                        let neighbor_luminance = neighbor_state.luminance;

                        if neighbor_luminance == 0 {
                            // No self-emission, darken it completely and continue propagation
                            self.set_block_light_level(level, &neighbor_pos, 0).ok();
                            self.queue_block_light_decrease(neighbor_pos, neighbor_light);
                        } else {
                            // Has self-emission, set to its own light and re-propagate from it
                            self.set_block_light_level(level, &neighbor_pos, neighbor_luminance)
                                .ok();
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

    pub fn check_block_light_updates(&self, level: &Arc<Level>, pos: BlockPos) {
        match level.lighting_config {
            LightingEngineConfig::Full => {
                self.set_block_light_level(level, &pos, 15).ok();
                return;
            }
            LightingEngineConfig::Dark => {
                self.set_block_light_level(level, &pos, 0).ok();
                return;
            }
            LightingEngineConfig::Default => {}
        }

        let current_light = self.get_block_light_level(level, &pos).unwrap_or(0);
        let block_state = level.get_block_state(&pos).to_state();
        let expected_light = block_state.luminance;

        // Handle light decrease (removing light source or placing opaque block)
        if expected_light < current_light {
            // Set to expected value immediately, then queue decrease to darken neighbors
            self.set_block_light_level(level, &pos, expected_light).ok();
            self.queue_block_light_decrease(pos, current_light);
            if expected_light > 0 {
                // BlockLightEngine re-adds a source after pulling its old light out. Without
                // this, changing (for example) a level-15 lamp to level 10 leaves its neighbors
                // permanently too dark after the decrease pass.
                self.queue_block_light_increase(pos, expected_light);
            }
        } else if expected_light > current_light {
            // Handle light increase (placing light source)
            self.set_block_light_level(level, &pos, expected_light).ok();
            self.queue_block_light_increase(pos, expected_light);
        }

        // Only check neighbors if we didn't trigger a decrease
        // Decrease propagation handles re-validating neighbors
        if expected_light >= current_light {
            self.check_neighbors_light_updates(level, pos, expected_light);
        }
    }

    pub fn check_neighbors_light_updates(
        &self,
        level: &Arc<Level>,
        pos: BlockPos,
        current_light: u8,
    ) {
        for dir in BlockDirection::all() {
            let neighbor_pos = pos.offset(dir.to_offset());
            if let Some(neighbor_light) = self.get_block_light_level(level, &neighbor_pos)
                && neighbor_light > current_light + 1
            {
                self.queue_block_light_increase(neighbor_pos, neighbor_light);
            }
        }
    }

    pub fn perform_sky_light_updates(&self, level: &Arc<Level>) -> i32 {
        let mut updates = 0;
        loop {
            let decrease_updates = self.perform_sky_light_decrease_updates(level);
            let increase_updates = self.perform_sky_light_increase_updates(level);

            updates += decrease_updates + increase_updates;

            if decrease_updates == 0 && increase_updates == 0 {
                break;
            }
        }
        updates
    }

    fn perform_sky_light_decrease_updates(&self, level: &Arc<Level>) -> i32 {
        let mut updates = 0;
        while let Some((pos, expected_light)) = self.sky_decrease.pop() {
            self.propagate_sky_light_decrease(level, &pos, expected_light);
            updates += 1;
        }
        updates
    }

    fn perform_sky_light_increase_updates(&self, level: &Arc<Level>) -> i32 {
        let mut updates = 0;
        while let Some((pos, expected_light)) = self.sky_increase.pop() {
            self.propagate_sky_light_increase(level, &pos, expected_light);
            updates += 1;
        }
        updates
    }

    fn propagate_sky_light_increase(&self, level: &Arc<Level>, pos: &BlockPos, light_level: u8) {
        for dir in BlockDirection::all() {
            let neighbor_pos = pos.offset(dir.to_offset());

            // Never propagate into an unloaded chunk. Writes to an unloaded
            // chunk are dropped silently, so the "brighter than neighbor" check
            // below would stay true forever and keep re-queuing the same
            // position, spinning this loop indefinitely at the border between
            // loaded and unloaded chunks.
            let (neighbor_chunk, _) = neighbor_pos.chunk_and_chunk_relative_position();
            if !level.is_chunk_loaded(&neighbor_chunk) {
                continue;
            }

            let neighbor_light = self.get_sky_light_level(level, &neighbor_pos);
            let neighbor_state = level.get_block_state(&neighbor_pos).to_state();
            let opacity = neighbor_state.opacity;

            // Calculate new light level for neighbor
            let new_light = sky_attenuation(light_level, opacity, dir == BlockDirection::Down);

            // Only propagate if new light is brighter than current light
            if new_light > neighbor_light {
                self.set_sky_light_level(level, &neighbor_pos, new_light)
                    .ok();

                if new_light > 0 {
                    self.queue_sky_light_increase(neighbor_pos, new_light);
                }
            }
        }
    }

    fn propagate_sky_light_decrease(&self, level: &Arc<Level>, pos: &BlockPos, removed_light: u8) {
        for dir in BlockDirection::all() {
            let neighbor_pos = pos.offset(dir.to_offset());

            // See `propagate_sky_light_increase`: skip unloaded chunks so sky
            // light updates never spin at loaded/unloaded chunk borders.
            let (neighbor_chunk, _) = neighbor_pos.chunk_and_chunk_relative_position();
            if !level.is_chunk_loaded(&neighbor_chunk) {
                continue;
            }

            let neighbor_light = self.get_sky_light_level(level, &neighbor_pos);
            if neighbor_light == 0 {
                continue; // Already dark
            }

            let neighbor_state = level.get_block_state(&neighbor_pos).to_state();
            let opacity = neighbor_state.opacity;

            // Calculate what we would have given this neighbor
            let expected = sky_attenuation(removed_light, opacity, dir == BlockDirection::Down);

            if neighbor_light == expected || neighbor_light < removed_light {
                // This neighbor was lit by us, darken it
                self.set_sky_light_level(level, &neighbor_pos, 0).ok();
                self.queue_sky_light_decrease(neighbor_pos, neighbor_light);
            } else if neighbor_light > removed_light {
                // Neighbor has brighter light from another source
                // Re-propagate from it to fill in the gap we left
                self.queue_sky_light_increase(neighbor_pos, neighbor_light);
            }
        }
    }

    pub fn check_sky_light_updates(&self, level: &Arc<Level>, pos: BlockPos) {
        if !Self::has_skylight(level) {
            return;
        }

        match level.lighting_config {
            LightingEngineConfig::Full => {
                self.set_sky_light_level(level, &pos, 15).ok();
                return;
            }
            LightingEngineConfig::Dark => {
                self.set_sky_light_level(level, &pos, 0).ok();
                return;
            }
            LightingEngineConfig::Default => {}
        }

        let current_light = self.get_sky_light_level(level, &pos);
        let block_state = level.get_block_state(&pos).to_state();
        let opacity = block_state.opacity;

        // Calculate expected sky light
        let expected_light = if opacity == 15 {
            // Fully opaque block = no light
            0
        } else {
            // Check if there's open sky above
            let has_sky = Self::has_open_sky_above(level, &pos);

            if has_sky {
                // Direct sunlight, reduced by opacity
                15u8.saturating_sub(opacity)
            } else {
                // No direct sky, check neighbors for best light. A neighbor
                // above transfers straight down (the direct-sun column case);
                // opacity is applied once per neighbor by `sky_attenuation`,
                // not again afterward.
                let mut best_light = 0;

                for dir in BlockDirection::all() {
                    let neighbor_pos = pos.offset(dir.to_offset());
                    let neighbor_light = self.get_sky_light_level(level, &neighbor_pos);
                    let potential =
                        sky_attenuation(neighbor_light, opacity, dir == BlockDirection::Up);
                    best_light = best_light.max(potential);
                }

                best_light
            }
        };

        // Update if needed
        if expected_light < current_light {
            // Light decreased
            self.set_sky_light_level(level, &pos, expected_light).ok();
            self.queue_sky_light_decrease(pos, current_light);
        } else if expected_light > current_light {
            // Light increased
            self.set_sky_light_level(level, &pos, expected_light).ok();
            self.queue_sky_light_increase(pos, expected_light);
        }

        // Notify neighbors if light increased or stayed same
        if expected_light >= current_light {
            self.check_neighbors_sky_light_updates(pos, expected_light);
        }
    }

    pub fn check_neighbors_sky_light_updates(&self, pos: BlockPos, current_light: u8) {
        // When we update a position, propagate to neighbors
        if current_light > 0 {
            self.queue_sky_light_increase(pos, current_light);
        }
    }

    pub fn get_block_light_level_sync(&self, level: &Level, position: &BlockPos) -> Option<u8> {
        let (chunk_pos, relative) = position.chunk_and_chunk_relative_position();

        level.read_chunk_sync(&chunk_pos, |chunk| {
            let section_idx = (relative.y - chunk.section.min_y) as usize / 16;
            let light_engine = chunk.light_engine.lock().ok()?;

            light_engine
                .block_light
                .get(section_idx)?
                .get(
                    relative.x as usize,
                    (relative.y - chunk.section.min_y) as usize % 16,
                    relative.z as usize,
                )
                .into()
        })?
    }

    pub fn get_sky_light_level_sync(&self, level: &Level, position: &BlockPos) -> u8 {
        if !Self::has_skylight(level) {
            return 0;
        }

        let (chunk_coordinate, relative) = position.chunk_and_chunk_relative_position();
        level
            .read_chunk_sync(&chunk_coordinate, |chunk| {
                let section_index =
                    (relative.y - chunk.section.min_y) as usize / BlockPalette::SIZE;

                let light_engine = chunk
                    .light_engine
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                // Bounds check for section index (lock the light engine)
                if section_index >= light_engine.sky_light.len() {
                    return 15;
                }

                light_engine.sky_light[section_index].get(
                    relative.x as usize,
                    (relative.y - chunk.section.min_y) as usize % BlockPalette::SIZE,
                    relative.z as usize,
                )
            })
            .unwrap_or(0)
    }

    pub fn get_block_light_level(&self, level: &Arc<Level>, position: &BlockPos) -> Option<u8> {
        let (chunk_pos, relative) = position.chunk_and_chunk_relative_position();

        level
            .read_chunk_sync(&chunk_pos, |chunk| {
                let section_idx = (relative.y - chunk.section.min_y) as usize / 16;
                chunk
                    .light_engine
                    .lock()
                    .ok()?
                    .block_light
                    .get(section_idx)
                    .map(|section| {
                        section.get(
                            relative.x as usize,
                            (relative.y - chunk.section.min_y) as usize % 16,
                            relative.z as usize,
                        )
                    })
            })
            .flatten()
    }

    pub fn set_block_light_level(
        &self,
        level: &Arc<Level>,
        position: &BlockPos,
        light_level: u8,
    ) -> Result<(), String> {
        let (chunk_coordinate, relative) = position.chunk_and_chunk_relative_position();
        let changed = level
            .read_chunk_sync(&chunk_coordinate, |chunk| {
                let section_index =
                    (relative.y - chunk.section.min_y) as usize / BlockPalette::SIZE;
                let relative_y = (relative.y - chunk.section.min_y) as usize % BlockPalette::SIZE;
                let mut light_engine = chunk
                    .light_engine
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                if section_index >= light_engine.block_light.len() {
                    return None;
                }

                let previous = light_engine.block_light[section_index].get(
                    relative.x as usize,
                    relative_y,
                    relative.z as usize,
                );
                if previous != light_level {
                    light_engine.block_light[section_index].set(
                        relative.x as usize,
                        relative_y,
                        relative.z as usize,
                        light_level,
                    );
                    // Mark chunk as dirty so lighting changes are saved to disk.
                    if !chunk.is_dirty() {
                        chunk.mark_dirty(true);
                    }
                }
                Some(previous != light_level)
            })
            .ok_or_else(|| "Chunk is not loaded".to_string())?
            .ok_or_else(|| "Invalid section index".to_string())?;
        if changed {
            self.mark_chunk_dirty(position);
        }
        Ok(())
    }

    pub fn get_sky_light_level(&self, level: &Arc<Level>, position: &BlockPos) -> u8 {
        if !Self::has_skylight(level) {
            return 0;
        }

        let (chunk_coordinate, relative) = position.chunk_and_chunk_relative_position();
        level
            .read_chunk_sync(&chunk_coordinate, |chunk| {
                let section_index =
                    (relative.y - chunk.section.min_y) as usize / BlockPalette::SIZE;

                let light_engine = chunk
                    .light_engine
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                // Bounds check for section index (lock the light engine)
                if section_index >= light_engine.sky_light.len() {
                    return 15;
                }

                light_engine.sky_light[section_index].get(
                    relative.x as usize,
                    (relative.y - chunk.section.min_y) as usize % BlockPalette::SIZE,
                    relative.z as usize,
                )
            })
            .unwrap_or(0)
    }

    pub fn set_sky_light_level(
        &self,
        level: &Arc<Level>,
        position: &BlockPos,
        light_level: u8,
    ) -> Result<(), String> {
        let (chunk_coordinate, relative) = position.chunk_and_chunk_relative_position();
        if !Self::has_skylight(level) {
            return Ok(());
        }

        let changed = level
            .read_chunk_sync(&chunk_coordinate, |chunk| {
                let section_index =
                    (relative.y - chunk.section.min_y) as usize / BlockPalette::SIZE;
                let relative_y = (relative.y - chunk.section.min_y) as usize % BlockPalette::SIZE;
                let mut light_engine = chunk
                    .light_engine
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                if section_index >= light_engine.sky_light.len() {
                    return None;
                }

                let previous = light_engine.sky_light[section_index].get(
                    relative.x as usize,
                    relative_y,
                    relative.z as usize,
                );
                if previous != light_level {
                    light_engine.sky_light[section_index].set(
                        relative.x as usize,
                        relative_y,
                        relative.z as usize,
                        light_level,
                    );
                    // Mark chunk as dirty so lighting changes are saved to disk.
                    if !chunk.is_dirty() {
                        chunk.mark_dirty(true);
                    }
                }
                Some(previous != light_level)
            })
            .ok_or_else(|| "Chunk is not loaded".to_string())?
            .ok_or_else(|| "Invalid section index".to_string())?;
        if changed {
            self.mark_chunk_dirty(position);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::DynamicLightEngine;
    use super::sky_attenuation;
    use pumpkin_util::math::vector2::Vector2;

    // vanilla: fromLevel - max(1, dampening) (LightEngine.getOpacity,
    // LightEngine.java:77-79; used at SkyLightEngine.java:152 and :188).
    #[test]
    fn transparent_block_decays_by_one() {
        assert_eq!(sky_attenuation(15, 0, false), 14);
    }

    #[test]
    fn dampening_one_decays_by_one_not_two() {
        // e.g. water, opacity 1 in pumpkin-data. A prior bug applied an extra
        // -1 on top of the opacity subtraction, decaying two levels here.
        assert_eq!(sky_attenuation(15, 1, false), 14);
    }

    #[test]
    fn dampening_above_one_decays_by_its_own_value() {
        assert_eq!(sky_attenuation(15, 5, false), 10);
    }

    #[test]
    fn direct_sun_column_does_not_decay_through_transparent_block() {
        assert_eq!(sky_attenuation(15, 0, true), 15);
    }

    #[test]
    fn direct_sun_column_still_decays_through_an_occluding_block() {
        // the no-decay column case is only for opacity == 0 (ChunkSkyLightSources
        // "source" concept); a block with real dampening must attenuate normally
        // even when it happens to sit directly below full sky light.
        assert_eq!(sky_attenuation(15, 1, true), 14);
    }

    #[test]
    fn saturates_at_zero() {
        assert_eq!(sky_attenuation(0, 5, false), 0);
    }

    #[test]
    fn dirty_chunks_are_deduplicated_when_drained() {
        let engine = DynamicLightEngine::new();
        engine.dirty_chunks.push(Vector2::new(2, -3));
        engine.dirty_chunks.push(Vector2::new(2, -3));
        engine.dirty_chunks.push(Vector2::new(-1, 4));

        let mut chunks = engine.take_dirty_chunks();
        chunks.sort_by_key(|chunk| (chunk.x, chunk.y));
        assert_eq!(chunks, vec![Vector2::new(-1, 4), Vector2::new(2, -3)]);
        assert!(engine.take_dirty_chunks().is_empty());
    }
}
