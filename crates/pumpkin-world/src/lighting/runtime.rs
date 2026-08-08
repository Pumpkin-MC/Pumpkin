use crate::chunk::io::Dirtiable;
use crate::chunk::palette::BlockPalette;
use crate::level::Level;
use crossbeam::queue::SegQueue;
use pumpkin_config::lighting::LightingEngineConfig;
use pumpkin_data::{Block, BlockDirection};
use pumpkin_util::math::{position::BlockPos, vector2::Vector2};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum PendingBorderLightUpdate {
    BlockDecrease(u8),
    BlockIncrease(u8, LightIncreaseMode),
    SkyDecrease(u8),
    SkyIncrease(u8, LightIncreaseMode),
}

impl PendingBorderLightUpdate {
    const fn phase(self) -> u8 {
        match self {
            Self::BlockDecrease(_) => 0,
            Self::BlockIncrease(_, _) => 1,
            Self::SkyDecrease(_) => 2,
            Self::SkyIncrease(_, _) => 3,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum LightIncreaseMode {
    All,
    Skip(BlockDirection),
    Only(BlockDirection),
}

impl LightIncreaseMode {
    fn allows(self, direction: BlockDirection) -> bool {
        match self {
            Self::All => true,
            Self::Skip(skip) => skip != direction,
            Self::Only(only) => only == direction,
        }
    }

    const fn sort_key(self) -> (u8, u8) {
        match self {
            Self::All => (0, 0),
            Self::Skip(direction) => (1, direction as u8),
            Self::Only(direction) => (2, direction as u8),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct PendingBorderUpdate {
    source_pos: BlockPos,
    target_pos: BlockPos,
    direction: BlockDirection,
    update: PendingBorderLightUpdate,
}

pub struct DynamicLightEngine {
    block_decrease: SegQueue<(BlockPos, u8, Option<BlockDirection>)>,
    block_increase: SegQueue<(BlockPos, u8, LightIncreaseMode)>,
    sky_decrease: SegQueue<(BlockPos, u8, Option<BlockDirection>)>,
    sky_increase: SegQueue<(BlockPos, u8, LightIncreaseMode)>,
    dirty_chunks: SegQueue<Vector2<i32>>,
    dirty_sections: Mutex<HashMap<Vector2<i32>, HashSet<usize>>>,
    pending_border_updates: Mutex<Vec<PendingBorderUpdate>>,
    processing: Mutex<()>,
}

impl DynamicLightEngine {
    #[must_use]
    pub fn new() -> Self {
        Self {
            block_decrease: SegQueue::new(),
            block_increase: SegQueue::new(),
            sky_decrease: SegQueue::new(),
            sky_increase: SegQueue::new(),
            dirty_chunks: SegQueue::new(),
            dirty_sections: Mutex::new(HashMap::new()),
            pending_border_updates: Mutex::new(Vec::new()),
            processing: Mutex::new(()),
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
    /// Finds the lowest direct-sky source in a column, matching
    /// `ChunkSkyLightSources.findLowestSourceY`. A source begins immediately
    /// above the first opaque or shape-occluded vertical edge.
    fn lowest_sky_source_y(
        level: &Arc<Level>,
        pos: &BlockPos,
        replacement: Option<(&BlockPos, &'static pumpkin_data::BlockState)>,
    ) -> i32 {
        let dimension = level.world_gen.dimension();
        let min_y = dimension.min_y;
        let max_y = min_y + dimension.height;
        let mut top_state = Block::AIR.default_state;
        let mut top_y = max_y;

        for y in (min_y..max_y).rev() {
            let current_pos = BlockPos::new(pos.0.x, y, pos.0.z);
            let state = replacement
                .filter(|(replacement_pos, _)| **replacement_pos == current_pos)
                .map_or_else(
                    || Self::state_for_lighting(level, &current_pos),
                    |(_, replacement_state)| replacement_state,
                );

            if state.opacity != 0
                || pumpkin_data::light_shape_occludes(top_state, state, BlockDirection::Down)
            {
                return top_y;
            }

            top_state = state;
            top_y = y;
        }

        // No edge was occluded: every block in the world is a direct-sky
        // source, just as ChunkSkyLightSources represents this below-world
        // sentinel.
        min_y - 1
    }

    fn update_sky_sources_for_change(
        &self,
        level: &Arc<Level>,
        pos: BlockPos,
        old_state: &'static pumpkin_data::BlockState,
        new_state: &'static pumpkin_data::BlockState,
    ) {
        if old_state == new_state || !Self::has_skylight(level) {
            return;
        }

        let old_source_y = Self::lowest_sky_source_y(level, &pos, Some((&pos, old_state)));
        let new_source_y = Self::lowest_sky_source_y(level, &pos, Some((&pos, new_state)));
        let min_y = level.world_gen.dimension().min_y;
        let max_y = min_y + level.world_gen.dimension().height;
        let old_start = old_source_y.clamp(min_y, max_y);
        let new_start = new_source_y.clamp(min_y, max_y);

        if old_start < new_start {
            // The direct-sky column moved upward. Remove the old level-15
            // source nodes before the normal decrease pass runs.
            for y in old_start..new_start {
                let source_pos = BlockPos::new(pos.0.x, y, pos.0.z);
                if self.get_sky_light_level(level, &source_pos) == 15 {
                    self.set_sky_light_level(level, &source_pos, 0).ok();
                    self.queue_sky_light_decrease(source_pos, 15);
                }
            }
        } else if new_start < old_start {
            // The column opened downward. Add the newly exposed source nodes.
            for y in new_start..old_start {
                let source_pos = BlockPos::new(pos.0.x, y, pos.0.z);
                self.set_sky_light_level(level, &source_pos, 15).ok();
                self.queue_sky_light_increase(source_pos, 15);
            }
        }
    }

    fn has_skylight(level: &Level) -> bool {
        level.world_gen.dimension().has_skylight
    }

    /// Vanilla's `LightEngine.getState` uses bedrock when the lighting source
    /// cannot provide a loaded chunk. Treating an unloaded neighbor as air
    /// would allow light to cross a chunk boundary that vanilla blocks.
    fn state_for_lighting(level: &Level, pos: &BlockPos) -> &'static pumpkin_data::BlockState {
        if level.is_chunk_loaded(&pos.chunk_position()) {
            level.get_block_state(pos).to_state()
        } else {
            Block::BEDROCK.default_state
        }
    }

    fn mark_chunk_dirty(&self, level: &Level, pos: &BlockPos) {
        let (chunk, _) = pos.chunk_and_chunk_relative_position();
        self.dirty_chunks.push(chunk);

        let min_y = level.world_gen.dimension().min_y;
        let section_count = level.world_gen.dimension().height as usize / BlockPalette::SIZE;
        if pos.0.y >= min_y {
            let section = ((pos.0.y - min_y) as usize) / BlockPalette::SIZE;
            if section < section_count {
                self.dirty_sections
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .entry(chunk)
                    .or_default()
                    .insert(section);
            }
        }
    }

    /// Returns and clears the changed physical light sections per chunk.
    pub fn take_dirty_sections(&self) -> Vec<(Vector2<i32>, Vec<usize>)> {
        while self.dirty_chunks.pop().is_some() {}
        let dirty = {
            let mut dirty = self
                .dirty_sections
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            std::mem::take(&mut *dirty)
        };
        dirty
            .into_iter()
            .map(|(chunk, sections)| {
                let mut sections: Vec<_> = sections.into_iter().collect();
                sections.sort_unstable();
                (chunk, sections)
            })
            .collect()
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
        let state = level.get_block_state(&pos).to_state();
        self.update_lighting_at_with_states(level, pos, state, state);
    }

    /// Handles a block replacement with the state pair that vanilla's light
    /// engine receives from `LevelChunk.setBlockState`. The pair matters for
    /// shape/opacity changes even when the block's emission is unchanged.
    pub fn update_lighting_at_with_states(
        &self,
        level: &Arc<Level>,
        pos: BlockPos,
        old_state: &'static pumpkin_data::BlockState,
        new_state: &'static pumpkin_data::BlockState,
    ) {
        let _processing = self
            .processing
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        self.queue_lighting_update_with_states_unlocked(level, pos, old_state, new_state);
        self.perform_lighting_updates_unlocked(level);
    }

    fn queue_lighting_update_with_states_unlocked(
        &self,
        level: &Arc<Level>,
        pos: BlockPos,
        old_state: &'static pumpkin_data::BlockState,
        new_state: &'static pumpkin_data::BlockState,
    ) {
        self.update_sky_sources_for_change(level, pos, old_state, new_state);
        self.check_block_light_updates_with_states(level, pos, old_state, new_state);
        self.check_sky_light_updates_with_states(level, pos, old_state, new_state);
    }

    /// Applies a group of block replacements under one serialized light-engine
    /// pass, matching vanilla's queued `checkBlock` work for structure updates.
    pub fn update_lighting_batch_with_states(
        &self,
        level: &Arc<Level>,
        changes: &[(
            BlockPos,
            pumpkin_data::BlockStateId,
            pumpkin_data::BlockStateId,
        )],
    ) {
        let _processing = self
            .processing
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for &(pos, old_state, new_state) in changes {
            self.queue_lighting_update_with_states_unlocked(
                level,
                pos,
                old_state.to_state(),
                new_state.to_state(),
            );
        }
        self.perform_lighting_updates_unlocked(level);
    }

    /// Drains queued block and sky light work until both layers converge.
    pub fn perform_lighting_updates(&self, level: &Arc<Level>) {
        let _processing = self
            .processing
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        self.perform_lighting_updates_unlocked(level);
    }

    fn perform_lighting_updates_unlocked(&self, level: &Arc<Level>) {
        self.perform_block_light_updates(level);
        self.perform_sky_light_updates(level);
    }

    fn defer_border_update(
        &self,
        source_pos: BlockPos,
        target_pos: BlockPos,
        direction: BlockDirection,
        update: PendingBorderLightUpdate,
    ) {
        let update = PendingBorderUpdate {
            source_pos,
            target_pos,
            direction,
            update,
        };
        let mut pending = self
            .pending_border_updates
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !pending.contains(&update) {
            pending.push(update);
        }
    }

    /// Replays changes whose propagation reached an unloaded neighboring
    /// chunk. This is the runtime equivalent of vanilla's light storage
    /// retaining section work until `LevelLightEngine` can see the chunk.
    #[expect(clippy::too_many_lines)]
    pub fn on_chunk_loaded(&self, level: &Arc<Level>, chunk_pos: Vector2<i32>) {
        let _processing = self
            .processing
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let pending = {
            let mut pending = self
                .pending_border_updates
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            std::mem::take(&mut *pending)
        };

        let mut retry = Vec::new();
        let mut deferred = Vec::new();
        for update in pending {
            let source_loaded = level.is_chunk_loaded(&update.source_pos.chunk_position());
            let target_loaded = level.is_chunk_loaded(&update.target_pos.chunk_position());
            if (update.source_pos.chunk_position() == chunk_pos
                || update.target_pos.chunk_position() == chunk_pos)
                && source_loaded
                && target_loaded
            {
                retry.push(update);
            } else {
                deferred.push(update);
            }
        }

        if !deferred.is_empty() {
            self.pending_border_updates
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .extend(deferred);
        }

        let had_retry = !retry.is_empty();
        // Vanilla drains decreases before increases. Keep the same phase order
        // when a border becomes available so a stale increase cannot overwrite
        // a removal merely because a set happened to iterate in another order.
        // Keep replay deterministic even when several changes hit the same edge
        // before the target chunk loads. Decreases always precede increases, and
        // equal-phase entries have a stable position/direction/level order.
        retry.sort_by_key(|update| {
            let (kind, level, mode) = match update.update {
                PendingBorderLightUpdate::BlockDecrease(level)
                | PendingBorderLightUpdate::SkyDecrease(level) => (0u8, level, (0, 0)),
                PendingBorderLightUpdate::BlockIncrease(level, mode)
                | PendingBorderLightUpdate::SkyIncrease(level, mode) => {
                    (1u8, level, mode.sort_key())
                }
            };
            (
                update.update.phase(),
                kind,
                update.source_pos.0.x,
                update.source_pos.0.y,
                update.source_pos.0.z,
                update.target_pos.0.x,
                update.target_pos.0.y,
                update.target_pos.0.z,
                update.direction.to_index(),
                level,
                mode.0,
                mode.1,
            )
        });
        for update in retry {
            match update.update {
                PendingBorderLightUpdate::BlockDecrease(light) => {
                    self.propagate_block_light_decrease_to(
                        level,
                        &update.source_pos,
                        &update.target_pos,
                        update.direction,
                        light,
                    );
                }
                PendingBorderLightUpdate::BlockIncrease(light, mode) => {
                    if !mode.allows(update.direction) {
                        continue;
                    }
                    self.propagate_block_light_increase_to(
                        level,
                        &update.source_pos,
                        &update.target_pos,
                        update.direction,
                        light,
                    );
                }
                PendingBorderLightUpdate::SkyDecrease(light) => {
                    self.propagate_sky_light_decrease_to(
                        level,
                        &update.source_pos,
                        &update.target_pos,
                        update.direction,
                        light,
                    );
                }
                PendingBorderLightUpdate::SkyIncrease(light, mode) => {
                    if !mode.allows(update.direction) {
                        continue;
                    }
                    self.propagate_sky_light_increase_to(
                        level,
                        &update.source_pos,
                        &update.target_pos,
                        update.direction,
                        light,
                    );
                }
            }
        }
        if had_retry {
            self.perform_lighting_updates_unlocked(level);
        }
    }

    pub fn queue_block_light_decrease(&self, pos: BlockPos, level: u8) {
        self.block_decrease.push((pos, level, None));
    }

    fn queue_block_light_decrease_skipping(
        &self,
        pos: BlockPos,
        level: u8,
        skip_direction: BlockDirection,
    ) {
        self.block_decrease.push((pos, level, Some(skip_direction)));
    }

    pub fn queue_block_light_increase(&self, pos: BlockPos, level: u8) {
        self.queue_block_light_increase_with_mode(pos, level, LightIncreaseMode::All);
    }

    fn queue_block_light_increase_with_mode(
        &self,
        pos: BlockPos,
        level: u8,
        mode: LightIncreaseMode,
    ) {
        self.block_increase.push((pos, level, mode));
    }

    fn queue_block_light_increase_only(&self, pos: BlockPos, level: u8, direction: BlockDirection) {
        self.queue_block_light_increase_with_mode(pos, level, LightIncreaseMode::Only(direction));
    }

    pub fn queue_sky_light_decrease(&self, pos: BlockPos, level: u8) {
        self.sky_decrease.push((pos, level, None));
    }

    fn queue_sky_light_decrease_skipping(
        &self,
        pos: BlockPos,
        level: u8,
        skip_direction: BlockDirection,
    ) {
        self.sky_decrease.push((pos, level, Some(skip_direction)));
    }

    pub fn queue_sky_light_increase(&self, pos: BlockPos, level: u8) {
        self.queue_sky_light_increase_with_mode(pos, level, LightIncreaseMode::All);
    }

    fn queue_sky_light_increase_with_mode(
        &self,
        pos: BlockPos,
        level: u8,
        mode: LightIncreaseMode,
    ) {
        self.sky_increase.push((pos, level, mode));
    }

    fn queue_sky_light_increase_only(&self, pos: BlockPos, level: u8, direction: BlockDirection) {
        self.queue_sky_light_increase_with_mode(pos, level, LightIncreaseMode::Only(direction));
    }

    fn perform_block_light_updates(&self, level: &Arc<Level>) -> i32 {
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

        while let Some((pos, expected_light, skip_direction)) = self.block_decrease.pop() {
            self.propagate_block_light_decrease(level, &pos, expected_light, skip_direction);
            updates += 1;
        }

        updates
    }

    fn perform_block_light_increase_updates(&self, level: &Arc<Level>) -> i32 {
        let mut updates = 0;

        while let Some((pos, expected_light, mode)) = self.block_increase.pop() {
            self.propagate_block_light_increase(level, &pos, expected_light, mode);
            updates += 1;
        }

        updates
    }

    fn propagate_block_light_increase(
        &self,
        level: &Arc<Level>,
        pos: &BlockPos,
        light_level: u8,
        mode: LightIncreaseMode,
    ) {
        for dir in BlockDirection::all() {
            if !mode.allows(dir) {
                continue;
            }
            let neighbor_pos = pos.offset(dir.to_offset());

            if !level.is_chunk_loaded(&neighbor_pos.chunk_position()) {
                self.defer_border_update(
                    *pos,
                    neighbor_pos,
                    dir,
                    PendingBorderLightUpdate::BlockIncrease(light_level, mode),
                );
                continue;
            }

            self.propagate_block_light_increase_to(level, pos, &neighbor_pos, dir, light_level);
        }
    }

    fn propagate_block_light_increase_to(
        &self,
        level: &Arc<Level>,
        pos: &BlockPos,
        neighbor_pos: &BlockPos,
        direction: BlockDirection,
        light_level: u8,
    ) {
        let from_state = Self::state_for_lighting(level, pos);
        if let Some(neighbor_light) = self.get_block_light_level(level, neighbor_pos) {
            let neighbor_state = Self::state_for_lighting(level, neighbor_pos);
            let opacity = neighbor_state.opacity.max(1);
            let new_light = light_level.saturating_sub(opacity);

            // Only propagate if new light is brighter than current light.
            if new_light > neighbor_light
                && !pumpkin_data::light_shape_occludes(from_state, neighbor_state, direction)
                && self
                    .set_block_light_level(level, neighbor_pos, new_light)
                    .is_ok()
                && new_light > 1
            {
                self.queue_block_light_increase_with_mode(
                    *neighbor_pos,
                    new_light,
                    LightIncreaseMode::Skip(direction.opposite()),
                );
            }
        }
    }

    fn propagate_block_light_decrease(
        &self,
        level: &Arc<Level>,
        pos: &BlockPos,
        removed_light_level: u8,
        skip_direction: Option<BlockDirection>,
    ) {
        if removed_light_level > 0 {
            // The source may still emit at a lower level; vanilla propagates the old level's
            // decrease first and then re-adds the block's remaining emission.
            for dir in BlockDirection::all() {
                if skip_direction == Some(dir) {
                    continue;
                }
                let neighbor_pos = pos.offset(dir.to_offset());

                if !level.is_chunk_loaded(&neighbor_pos.chunk_position()) {
                    self.defer_border_update(
                        *pos,
                        neighbor_pos,
                        dir,
                        PendingBorderLightUpdate::BlockDecrease(removed_light_level),
                    );
                    continue;
                }

                self.propagate_block_light_decrease_to(
                    level,
                    pos,
                    &neighbor_pos,
                    dir,
                    removed_light_level,
                );
            }
        }
    }

    fn propagate_block_light_decrease_to(
        &self,
        level: &Arc<Level>,
        _pos: &BlockPos,
        neighbor_pos: &BlockPos,
        direction: BlockDirection,
        removed_light_level: u8,
    ) {
        if let Some(neighbor_light) = self.get_block_light_level(level, neighbor_pos) {
            if neighbor_light == 0 {
                return; // Skip if already 0
            }

            // Vanilla compares the stored level with the level that can arrive
            // from the removed node. It does not recompute the target's opacity
            // here; that opacity was already applied by the increase path.
            if neighbor_light <= removed_light_level.saturating_sub(1) {
                let neighbor_luminance = Self::state_for_lighting(level, neighbor_pos).luminance;
                self.set_block_light_level(level, neighbor_pos, 0).ok();
                if neighbor_luminance < neighbor_light {
                    self.queue_block_light_decrease_skipping(
                        *neighbor_pos,
                        neighbor_light,
                        direction.opposite(),
                    );
                }
                if neighbor_luminance > 0 {
                    self.queue_block_light_increase(*neighbor_pos, neighbor_luminance);
                }
            } else {
                // This neighbor has brighter light from another source.
                self.queue_block_light_increase_only(
                    *neighbor_pos,
                    neighbor_light,
                    direction.opposite(),
                );
            }
        }
    }

    pub fn check_block_light_updates(&self, level: &Arc<Level>, pos: BlockPos) {
        let state = level.get_block_state(&pos).to_state();
        self.check_block_light_updates_with_states(level, pos, state, state);
    }

    fn check_block_light_updates_with_states(
        &self,
        level: &Arc<Level>,
        pos: BlockPos,
        old_state: &'static pumpkin_data::BlockState,
        block_state: &'static pumpkin_data::BlockState,
    ) {
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
        let expected_light = block_state.luminance;
        let properties_changed = old_state != block_state
            && (old_state.opacity != block_state.opacity
                || old_state.luminance != block_state.luminance
                || old_state.use_shape_for_light_occlusion
                || block_state.use_shape_for_light_occlusion);

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
        } else if properties_changed {
            // This is vanilla's PULL_LIGHT_IN_ENTRY: a shape or opacity
            // replacement invalidates paths even when emission is unchanged.
            self.queue_block_light_decrease(pos, 1);
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

    fn perform_sky_light_updates(&self, level: &Arc<Level>) -> i32 {
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
        while let Some((pos, expected_light, skip_direction)) = self.sky_decrease.pop() {
            self.propagate_sky_light_decrease(level, &pos, expected_light, skip_direction);
            updates += 1;
        }
        updates
    }

    fn perform_sky_light_increase_updates(&self, level: &Arc<Level>) -> i32 {
        let mut updates = 0;
        while let Some((pos, expected_light, mode)) = self.sky_increase.pop() {
            self.propagate_sky_light_increase(level, &pos, expected_light, mode);
            updates += 1;
        }
        updates
    }

    fn propagate_sky_light_increase(
        &self,
        level: &Arc<Level>,
        pos: &BlockPos,
        light_level: u8,
        mode: LightIncreaseMode,
    ) {
        for dir in BlockDirection::all() {
            if !mode.allows(dir) {
                continue;
            }
            let neighbor_pos = pos.offset(dir.to_offset());

            // Never propagate into an unloaded chunk. Writes to an unloaded
            // chunk are dropped silently, so the "brighter than neighbor" check
            // below would stay true forever and keep re-queuing the same
            // position, spinning this loop indefinitely at the border between
            // loaded and unloaded chunks.
            let (neighbor_chunk, _) = neighbor_pos.chunk_and_chunk_relative_position();
            if !level.is_chunk_loaded(&neighbor_chunk) {
                self.defer_border_update(
                    *pos,
                    neighbor_pos,
                    dir,
                    PendingBorderLightUpdate::SkyIncrease(light_level, mode),
                );
                continue;
            }

            self.propagate_sky_light_increase_to(level, pos, &neighbor_pos, dir, light_level);
        }
    }

    fn propagate_sky_light_increase_to(
        &self,
        level: &Arc<Level>,
        pos: &BlockPos,
        neighbor_pos: &BlockPos,
        direction: BlockDirection,
        light_level: u8,
    ) {
        let from_state = Self::state_for_lighting(level, pos);
        let neighbor_light = self.get_sky_light_level(level, neighbor_pos);
        let neighbor_state = Self::state_for_lighting(level, neighbor_pos);
        let opacity = neighbor_state.opacity;

        // Calculate new light level for neighbor.
        let new_light = sky_attenuation(light_level, opacity, direction == BlockDirection::Down);

        // Only propagate if new light is brighter than current light.
        if new_light > neighbor_light
            && !pumpkin_data::light_shape_occludes(from_state, neighbor_state, direction)
        {
            self.set_sky_light_level(level, neighbor_pos, new_light)
                .ok();

            if new_light > 0 {
                self.queue_sky_light_increase_with_mode(
                    *neighbor_pos,
                    new_light,
                    LightIncreaseMode::Skip(direction.opposite()),
                );
            }
        }
    }

    fn propagate_sky_light_decrease(
        &self,
        level: &Arc<Level>,
        pos: &BlockPos,
        removed_light: u8,
        skip_direction: Option<BlockDirection>,
    ) {
        for dir in BlockDirection::all() {
            if skip_direction == Some(dir) {
                continue;
            }
            let neighbor_pos = pos.offset(dir.to_offset());

            // See `propagate_sky_light_increase`: skip unloaded chunks so sky
            // light updates never spin at loaded/unloaded chunk borders.
            let (neighbor_chunk, _) = neighbor_pos.chunk_and_chunk_relative_position();
            if !level.is_chunk_loaded(&neighbor_chunk) {
                self.defer_border_update(
                    *pos,
                    neighbor_pos,
                    dir,
                    PendingBorderLightUpdate::SkyDecrease(removed_light),
                );
                continue;
            }

            self.propagate_sky_light_decrease_to(level, pos, &neighbor_pos, dir, removed_light);
        }
    }

    fn propagate_sky_light_decrease_to(
        &self,
        level: &Arc<Level>,
        _pos: &BlockPos,
        neighbor_pos: &BlockPos,
        direction: BlockDirection,
        removed_light: u8,
    ) {
        let neighbor_light = self.get_sky_light_level(level, neighbor_pos);
        if neighbor_light == 0 {
            return; // Already dark
        }

        if neighbor_light <= removed_light.saturating_sub(1) {
            // This neighbor was lit by us, darken it.
            self.set_sky_light_level(level, neighbor_pos, 0).ok();
            self.queue_sky_light_decrease_skipping(
                *neighbor_pos,
                neighbor_light,
                direction.opposite(),
            );
        } else {
            // Neighbor has light that cannot have come only from the removed
            // node. Re-propagate it to fill the gap.
            self.queue_sky_light_increase_only(*neighbor_pos, neighbor_light, direction.opposite());
        }
    }

    pub fn check_sky_light_updates(&self, level: &Arc<Level>, pos: BlockPos) {
        let state = level.get_block_state(&pos).to_state();
        self.check_sky_light_updates_with_states(level, pos, state, state);
    }

    fn check_sky_light_updates_with_states(
        &self,
        level: &Arc<Level>,
        pos: BlockPos,
        old_state: &'static pumpkin_data::BlockState,
        block_state: &'static pumpkin_data::BlockState,
    ) {
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
        let opacity = block_state.opacity;
        let properties_changed = old_state != block_state
            && (old_state.opacity != block_state.opacity
                || old_state.use_shape_for_light_occlusion
                || block_state.use_shape_for_light_occlusion);

        // Calculate expected sky light
        let source_y = Self::lowest_sky_source_y(level, &pos, None);
        let expected_light = if pos.0.y >= source_y {
            15
        } else if opacity == 15 {
            // Fully opaque block = no light
            0
        } else {
            // No direct sky, check neighbors for best light. A neighbor above
            // transfers straight down through a transparent block; opacity is
            // applied once per neighbor by `sky_attenuation`.
            let mut best_light = 0;

            for dir in BlockDirection::all() {
                let neighbor_pos = pos.offset(dir.to_offset());
                let neighbor_light = self.get_sky_light_level(level, &neighbor_pos);
                let neighbor_state = Self::state_for_lighting(level, &neighbor_pos);
                if pumpkin_data::light_shape_occludes(neighbor_state, block_state, dir.opposite()) {
                    continue;
                }
                let potential = sky_attenuation(neighbor_light, opacity, dir == BlockDirection::Up);
                best_light = best_light.max(potential);
            }

            best_light
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
        } else if properties_changed {
            self.queue_sky_light_decrease(pos, 1);
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
            self.mark_chunk_dirty(level, position);
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
            self.mark_chunk_dirty(level, position);
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
