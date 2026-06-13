use crate::block::RawBlockState;
use crate::chunk::io::Dirtiable;
use crate::chunk::palette::BlockPalette;
use crate::level::Level;
use crossbeam::queue::SegQueue;
use pumpkin_config::lighting::LightingEngineConfig;
use pumpkin_data::BlockDirection;
use pumpkin_util::math::position::BlockPos;
use rustc_hash::FxHashMap;
use std::sync::Arc;

pub struct DynamicLightEngine {
    block_decrease: SegQueue<(BlockPos, u8)>,
    block_increase: SegQueue<(BlockPos, u8)>,
    sky_decrease: SegQueue<(BlockPos, u8)>,
    sky_increase: SegQueue<(BlockPos, u8)>,
}

impl DynamicLightEngine {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            block_decrease: SegQueue::new(),
            block_increase: SegQueue::new(),
            sky_decrease: SegQueue::new(),
            sky_increase: SegQueue::new(),
        }
    }
}
impl Default for DynamicLightEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl DynamicLightEngine {
    /// Checks if there is an open sky above the given position (no opaque blocks blocking sky light).
    fn has_open_sky_above(level: &Arc<Level>, pos: &BlockPos) -> bool {
        let max_y = 319; // Maximum build height in Minecraft, can be adjusted if needed
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

    /// Updates lighting for many block changes at once.
    ///
    /// Equivalent in result to calling [`Self::update_lighting_at`] for every
    /// position, but the expensive propagation-to-convergence pass runs **once**
    /// for the whole batch instead of once per block. A per-block drain re-walks
    /// the same overlapping light fields N times; for a large paste (hundreds of
    /// thousands of blocks) that turns a sub-second relight into tens of seconds.
    /// Here we seed every position's check first, then drain the shared queues a
    /// single time.
    pub fn update_lighting_bulk<I>(&self, level: &Arc<Level>, positions: I)
    where
        I: IntoIterator<Item = BlockPos>,
    {
        // Seed all block-light + sky-light checks before draining either queue.
        // The check_* calls only enqueue work (and set the source cell); they do
        // not propagate, so order between positions does not matter.
        //
        // `has_open_sky_above` walks the whole column above a position one
        // `get_block_state` (chunk-map lookup) at a time. A paste touches many
        // positions in the same (x, z) column, so instead find the highest
        // opaque block once per column — with a single chunk read — and answer
        // the open-sky question for every position in that column from it.
        let mut highest_opaque_by_column: FxHashMap<(i32, i32), i32> = FxHashMap::default();
        for pos in positions {
            self.check_block_light_updates(level, pos);

            let column = (pos.0.x, pos.0.z);
            let highest_opaque = *highest_opaque_by_column
                .entry(column)
                .or_insert_with(|| Self::highest_opaque_y(level, column.0, column.1));
            self.check_sky_light_updates_with(level, pos, || pos.0.y >= highest_opaque);
        }

        // Drain to convergence once for the entire batch.
        self.perform_block_light_updates(level);
        self.perform_sky_light_updates(level);
    }

    /// The Y of the highest block with non-zero opacity in the given column, or
    /// `i32::MIN` when the column has none (open to the sky at every height).
    ///
    /// A position has open sky above (in the sense of [`Self::has_open_sky_above`])
    /// exactly when its Y is at or above this value. Unloaded chunks read as
    /// all-transparent, matching the per-position scan.
    fn highest_opaque_y(level: &Arc<Level>, block_x: i32, block_z: i32) -> i32 {
        let probe = BlockPos::new(block_x, 0, block_z);
        let (chunk_coordinate, relative) = probe.chunk_and_chunk_relative_position();
        let max_y = 319;

        level
            .read_chunk_sync(&chunk_coordinate, |chunk| {
                for y in (chunk.section.min_y..=max_y).rev() {
                    let Some(state_id) = chunk.section.get_block_absolute_y(
                        relative.x as usize,
                        y,
                        relative.z as usize,
                    ) else {
                        continue;
                    };
                    if RawBlockState(state_id).to_state().opacity > 0 {
                        return y;
                    }
                }
                i32::MIN
            })
            .unwrap_or(i32::MIN)
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

    /// Drains `queue` to empty, always processing the entry with the highest
    /// light level first.
    ///
    /// Propagation never enqueues an entry brighter than its source, so
    /// brightest-first order means every cell is written its final value the
    /// first time a wave reaches it. FIFO order lets a dim wavefront raise a
    /// cell repeatedly (up to 15 times) as brighter waves arrive late, which
    /// turns a large paste's shadow refill into minutes of redundant work.
    fn drain_highest_first(
        queue: &SegQueue<(BlockPos, u8)>,
        mut propagate: impl FnMut(&BlockPos, u8),
    ) -> i32 {
        const LEVELS: usize = 16;
        let mut buckets: [Vec<(BlockPos, u8)>; LEVELS] = Default::default();
        let mut updates = 0;

        loop {
            while let Some((pos, light)) = queue.pop() {
                buckets[usize::from(light.min(15))].push((pos, light));
            }

            let Some((pos, light)) = buckets.iter_mut().rev().find_map(|bucket| bucket.pop())
            else {
                break;
            };

            propagate(&pos, light);
            updates += 1;
        }

        updates
    }

    fn perform_block_light_decrease_updates(&self, level: &Arc<Level>) -> i32 {
        Self::drain_highest_first(&self.block_decrease, |pos, expected_light| {
            self.propagate_block_light_decrease(level, pos, expected_light);
        })
    }

    fn perform_block_light_increase_updates(&self, level: &Arc<Level>) -> i32 {
        Self::drain_highest_first(&self.block_increase, |pos, expected_light| {
            self.propagate_block_light_increase(level, pos, expected_light);
        })
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
        // Check what the current light level actually is at this position
        let current_level = self.get_block_light_level(level, pos).unwrap_or(0);

        // Only propagate decrease if this position hasn't already been reset to 0
        // This prevents positions that were intentionally set to 0 from propagating light
        if current_level == 0 && removed_light_level > 0 {
            // This position was already darkened, so we propagate the darkness to neighbors
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
        Self::drain_highest_first(&self.sky_decrease, |pos, expected_light| {
            self.propagate_sky_light_decrease(level, pos, expected_light);
        })
    }

    fn perform_sky_light_increase_updates(&self, level: &Arc<Level>) -> i32 {
        Self::drain_highest_first(&self.sky_increase, |pos, expected_light| {
            self.propagate_sky_light_increase(level, pos, expected_light);
        })
    }

    fn propagate_sky_light_increase(&self, level: &Arc<Level>, pos: &BlockPos, light_level: u8) {
        for dir in BlockDirection::all() {
            let neighbor_pos = pos.offset(dir.to_offset());

            let neighbor_light = self.get_sky_light_level(level, &neighbor_pos);
            let neighbor_state = level.get_block_state(&neighbor_pos).to_state();
            let opacity = neighbor_state.opacity;

            // Calculate new light level for neighbor
            let new_light = if light_level == 15 && dir == BlockDirection::Down && opacity == 0 {
                // Special case: Sky light at 15 propagates down as 15 through transparent blocks
                15
            } else {
                // Normal propagation: reduce by 1 for distance, then by opacity
                light_level.saturating_sub(1).saturating_sub(opacity)
            };

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

            let neighbor_light = self.get_sky_light_level(level, &neighbor_pos);
            if neighbor_light == 0 {
                continue; // Already dark
            }

            let neighbor_state = level.get_block_state(&neighbor_pos).to_state();
            let opacity = neighbor_state.opacity;

            // Calculate what we would have given this neighbor
            let expected = if removed_light == 15 && dir == BlockDirection::Down && opacity == 0 {
                15
            } else {
                removed_light.saturating_sub(1).saturating_sub(opacity)
            };

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
        self.check_sky_light_updates_with(level, pos, || Self::has_open_sky_above(level, &pos));
    }

    /// [`Self::check_sky_light_updates`] with the open-sky test supplied by the
    /// caller, so bulk updates can answer it from a per-column cache instead of
    /// rescanning the column above every position.
    fn check_sky_light_updates_with(
        &self,
        level: &Arc<Level>,
        pos: BlockPos,
        has_open_sky_above: impl FnOnce() -> bool,
    ) {
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
            let has_sky = has_open_sky_above();

            if has_sky {
                // Direct sunlight, reduced by opacity
                15u8.saturating_sub(opacity)
            } else {
                // No direct sky, check neighbors for best light
                let mut best_light = 0;

                for dir in BlockDirection::all() {
                    let neighbor_pos = pos.offset(dir.to_offset());

                    let neighbor_light = self.get_sky_light_level(level, &neighbor_pos);
                    // Calculate potential light from this neighbor
                    let potential = if neighbor_light == 15 && dir == BlockDirection::Up {
                        // Sky light at 15 from above stays 15
                        15
                    } else {
                        // Normal decay
                        neighbor_light.saturating_sub(1)
                    };

                    best_light = best_light.max(potential);
                }

                // Apply opacity to the best incoming light
                best_light.saturating_sub(opacity)
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

        // Increases were already queued above; unchanged lit cells still need to seed neighbors.
        if expected_light == current_light {
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
            let relative_y = (relative.y - chunk.section.min_y) as usize;
            let section_idx = relative_y / BlockPalette::SIZE;
            let light_engine = chunk.light_engine.lock().ok()?;

            light_engine
                .block_light
                .get(section_idx)?
                .get(
                    relative.x as usize,
                    relative_y % BlockPalette::SIZE,
                    relative.z as usize,
                )
                .into()
        })?
    }

    pub fn get_sky_light_level_sync(&self, level: &Level, position: &BlockPos) -> u8 {
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
                let relative_y = (relative.y - chunk.section.min_y) as usize;
                let section_idx = relative_y / BlockPalette::SIZE;
                let light_engine = chunk.light_engine.lock().ok()?;

                let light_level = light_engine.block_light.get(section_idx)?.get(
                    relative.x as usize,
                    relative_y % BlockPalette::SIZE,
                    relative.z as usize,
                );

                Some(light_level)
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
        level.read_chunk_sync(&chunk_coordinate, |chunk| {
            let section_index = (relative.y - chunk.section.min_y) as usize / BlockPalette::SIZE;
            // Bounds check for section index
            let mut light_engine = chunk
                .light_engine
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
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
        });
        Ok(())
    }

    pub fn get_sky_light_level(&self, level: &Arc<Level>, position: &BlockPos) -> u8 {
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
        level.read_chunk_sync(&chunk_coordinate, |chunk| {
            let section_index = (relative.y - chunk.section.min_y) as usize / BlockPalette::SIZE;
            // Bounds check for section index
            let mut light_engine = chunk
                .light_engine
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
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
        });
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    };
    use std::time::{Duration, Instant};

    use pumpkin_data::{Block, chunk::ChunkStatus, fluid::Fluid};
    use pumpkin_util::math::{position::BlockPos, vector2::Vector2};
    use rustc_hash::FxHashMap;
    use temp_dir::TempDir;

    use crate::chunk::format::LightContainer;
    use crate::chunk::{ChunkData, ChunkHeightmaps, ChunkLight, ChunkSections};
    use crate::level::Level;
    use crate::tick::scheduler::ChunkTickScheduler;

    const MIN_Y: i32 = -64;
    const SECTION_COUNT: usize = 24;
    const START_Y: i32 = 64;

    #[derive(Clone, Copy)]
    struct BenchConfig {
        width: i32,
        height: i32,
        depth: i32,
        runs: usize,
        margin: i32,
        mode: BenchMode,
    }

    #[derive(Clone, Copy, Debug)]
    enum BenchMode {
        Block,
        Sky,
    }

    impl BenchConfig {
        fn from_env() -> Self {
            let mode = BenchMode::from_env();
            Self {
                width: env_i32("PUMPKIN_LIGHT_BENCH_WIDTH", 64),
                height: env_i32("PUMPKIN_LIGHT_BENCH_HEIGHT", 58),
                depth: env_i32("PUMPKIN_LIGHT_BENCH_DEPTH", 64),
                runs: env_usize("PUMPKIN_LIGHT_BENCH_RUNS", 1),
                margin: env_i32(
                    "PUMPKIN_LIGHT_BENCH_MARGIN",
                    match mode {
                        BenchMode::Block => 16,
                        BenchMode::Sky => 0,
                    },
                ),
                mode,
            }
        }

        fn block_count(self) -> usize {
            (self.width * self.height * self.depth) as usize
        }
    }

    impl BenchMode {
        fn from_env() -> Self {
            match std::env::var("PUMPKIN_LIGHT_BENCH_MODE")
                .unwrap_or_else(|_| "block".to_string())
                .to_ascii_lowercase()
                .as_str()
            {
                "sky" => Self::Sky,
                "block" => Self::Block,
                other => panic!("unknown PUMPKIN_LIGHT_BENCH_MODE {other:?}; use block or sky"),
            }
        }

        fn sky_default(self) -> u8 {
            match self {
                Self::Block => 0,
                Self::Sky => 15,
            }
        }

        fn placed_state(self) -> u16 {
            match self {
                Self::Block => Block::GLOWSTONE.default_state.id,
                Self::Sky => Block::STONE.default_state.id,
            }
        }
    }

    fn env_i32(name: &str, default: i32) -> i32 {
        std::env::var(name)
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(default)
    }

    fn env_usize(name: &str, default: usize) -> usize {
        std::env::var(name)
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(default)
    }

    fn make_chunk(chunk_x: i32, chunk_z: i32, sky_default: u8) -> Arc<ChunkData> {
        Arc::new(ChunkData {
            section: ChunkSections::new(SECTION_COUNT, MIN_Y),
            heightmap: Mutex::new(ChunkHeightmaps::default()),
            x: chunk_x,
            z: chunk_z,
            block_ticks: ChunkTickScheduler::<&'static Block>::default(),
            fluid_ticks: ChunkTickScheduler::<&'static Fluid>::default(),
            pending_block_entities: Mutex::new(FxHashMap::default()),
            light_engine: Mutex::new(ChunkLight {
                sky_light: vec![LightContainer::new_empty(sky_default); SECTION_COUNT]
                    .into_boxed_slice(),
                block_light: vec![LightContainer::new_empty(0); SECTION_COUNT].into_boxed_slice(),
            }),
            light_populated: AtomicBool::new(true),
            status: ChunkStatus::Full,
            blending_data: None,
            dirty: AtomicBool::new(false),
        })
    }

    fn install_chunks(
        level: &Arc<Level>,
        min_x: i32,
        max_x: i32,
        min_z: i32,
        max_z: i32,
        sky_default: u8,
    ) {
        level.loaded_chunks.clear();

        let min_chunk_x = min_x.div_euclid(16);
        let max_chunk_x = max_x.div_euclid(16);
        let min_chunk_z = min_z.div_euclid(16);
        let max_chunk_z = max_z.div_euclid(16);

        for chunk_x in min_chunk_x..=max_chunk_x {
            for chunk_z in min_chunk_z..=max_chunk_z {
                level.loaded_chunks.insert(
                    Vector2::new(chunk_x, chunk_z),
                    make_chunk(chunk_x, chunk_z, sky_default),
                );
            }
        }
    }

    fn prepare_case(level: &Arc<Level>, config: BenchConfig) -> Vec<BlockPos> {
        let start_x = -(config.width / 2);
        let start_z = -(config.depth / 2);
        let end_x = start_x + config.width - 1;
        let end_y = START_Y + config.height - 1;
        let end_z = start_z + config.depth - 1;

        install_chunks(
            level,
            start_x - config.margin,
            end_x + config.margin,
            start_z - config.margin,
            end_z + config.margin,
            config.mode.sky_default(),
        );

        let placed_state = config.mode.placed_state();
        let mut positions = Vec::with_capacity(config.block_count());

        for y in START_Y..=end_y {
            for z in start_z..=end_z {
                for x in start_x..=end_x {
                    let pos = BlockPos::new(x, y, z);
                    level.set_block_state(&pos, placed_state);
                    positions.push(pos);
                }
            }
        }

        positions
    }

    fn format_duration(duration: Duration) -> String {
        format!("{:.3}s", duration.as_secs_f64())
    }

    #[test]
    #[ignore = "run manually to measure runtime bulk lighting speed"]
    fn bulk_lighting_speed() {
        let config = BenchConfig::from_env();
        assert!(config.width > 0 && config.height > 0 && config.depth > 0);
        assert!(config.runs > 0);
        assert!(START_Y + config.height <= MIN_Y + SECTION_COUNT as i32 * 16);

        let temp_dir = TempDir::new().expect("temporary benchmark world");
        let level = Level::new_for_lighting_bench(temp_dir.path().to_path_buf());

        println!(
            "bulk lighting speed case: mode={:?}, {}x{}x{} = {} positions, margin={} blocks, {} run(s)",
            config.mode,
            config.width,
            config.height,
            config.depth,
            config.block_count(),
            config.margin,
            config.runs
        );

        let mut total = Duration::ZERO;
        for run in 1..=config.runs {
            let positions = prepare_case(&level, config);

            let start = Instant::now();
            level.light_engine.update_lighting_bulk(&level, positions);
            let elapsed = start.elapsed();

            total += elapsed;
            println!("run {run}: {}", format_duration(elapsed));
        }

        println!(
            "average: {}",
            format_duration(total / u32::try_from(config.runs).unwrap())
        );

        level.cancel_token.cancel();
        level.should_unload.store(true, Ordering::Relaxed);
        level.shut_down_chunk_system.store(true, Ordering::Relaxed);
    }
}
