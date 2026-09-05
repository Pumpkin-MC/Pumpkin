use crate::chunk::palette::BlockPalette;
use crate::chunk::{ChunkData, ChunkHeightmapType};
use crate::level::Level;
use crate::lighting::chunk_access::{ChunkCursor, VerticalInChunk};
use crate::lighting::decayed;
use crate::lighting::occlusion;
use crate::lighting::sky_light_height::{SkyLightHeight, SkyLightHeightMigration, SkyLightTier};
use crate::lighting::stats::{Counter, LightCounters, LightPassStats, LocalCounters};
use crossbeam::queue::SegQueue;
use pumpkin_config::lighting::LightingEngineConfig;
use pumpkin_data::BlockDirection;
use pumpkin_data::BlockStateId;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector2::Vector2;
use rustc_hash::FxHashSet;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tracing::debug;

/// One `drain_queued` slice per `ServerLevel.tick`. Vanilla `LightEngine.runLightUpdates`
/// empties the queues; `ThreadedLevelLightEngine` does that on the light thread.
/// Leftover is visible as delayed shadows after mining, placing, or a chunk-border sky refill.
const LIGHT_UPDATES_PER_PASS: i32 = 16_384;

pub struct DynamicLightEngine {
    block_decrease: SegQueue<(BlockPos, u8)>,
    block_increase: SegQueue<(BlockPos, u8)>,
    sky_decrease: SegQueue<(BlockPos, u8)>,
    sky_increase: SegQueue<(BlockPos, u8)>,
    /// Positions whose light has to be re-derived. Vanilla `LightEngine.blockNodesToCheck`
    nodes_to_check: SegQueue<BlockPos>,
    /// Serialises the flood, and only the flood: two concurrent [`Self::drain_queued`]
    /// would ping-pong between the decrease and the increase loop and never settle.
    propagate_lock: Mutex<()>,
    counters: LightCounters,
}

impl DynamicLightEngine {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            block_decrease: SegQueue::new(),
            block_increase: SegQueue::new(),
            sky_decrease: SegQueue::new(),
            sky_increase: SegQueue::new(),
            nodes_to_check: SegQueue::new(),
            propagate_lock: Mutex::new(()),
            counters: LightCounters::new(),
        }
    }
}
impl Default for DynamicLightEngine {
    fn default() -> Self {
        Self::new()
    }
}
impl DynamicLightEngine {
    /// Open sky above `pos`. Scan only to `WorldSurface` (air above it).
    fn has_open_sky_above(cursor: &mut ChunkCursor, pos: &BlockPos) -> bool {
        cursor.counters.bump(Counter::SkyColumnScan);
        let Some(chunk) = cursor.chunk_for(pos) else {
            return false;
        };
        let min_y = chunk.section.min_y;
        let (_, relative) = pos.chunk_and_chunk_relative_position();
        let (local_x, local_z) = (relative.x as usize, relative.z as usize);

        let surface = {
            let heightmap = chunk
                .heightmap
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            heightmap
                .get(
                    ChunkHeightmapType::WorldSurface,
                    relative.x,
                    relative.z,
                    min_y,
                )
                .min(min_y + SkyLightHeight::chunk_height(chunk) - 1)
        };
        if surface <= pos.0.y {
            return true;
        }

        let (blocked, reads) = chunk.section.with_blocks(|sections| {
            let mut reads = 0u64;
            for y in (pos.0.y + 1)..=surface {
                reads += 1;
                let rel_y = (y - min_y) as usize;
                let opacity = sections
                    .get(rel_y / BlockPalette::SIZE)
                    .map_or(0, |section| {
                        crate::lighting::opacity_of(section.get(
                            local_x,
                            rel_y % BlockPalette::SIZE,
                            local_z,
                        ))
                    });
                if opacity > 0 {
                    return (true, reads);
                }
            }
            (false, reads)
        });
        cursor.counters.bump_n(Counter::SkyColumnRead, reads);

        !blocked
    }

    /// 3-Tier culling for the open-sky question, backed by the cached per-chunk cut height.
    /// Only Tier 3 pays for [`Self::has_open_sky_above`]; the other two answer from 24 bits.
    fn sky_tier(cursor: &mut ChunkCursor, pos: &BlockPos) -> SkyLightTier {
        let (chunk_pos, relative) = pos.chunk_and_chunk_relative_position();
        let Some((tier, height)) = cursor.chunk_at(chunk_pos).map(|chunk| {
            let height = SkyLightHeightMigration::get(chunk);
            let tier = height.tier(
                pos.0.y,
                relative.x,
                relative.z,
                chunk.section.min_y,
                SkyLightHeight::chunk_height(chunk),
            );
            (tier, height)
        }) else {
            return SkyLightTier::Unknown;
        };

        if tier == SkyLightTier::Unknown {
            return tier;
        }
        // Skip cursor: neighbour would evict the memo.
        if Self::border_sides_agree(cursor.level, chunk_pos, relative.x, relative.z, height) {
            tier
        } else {
            SkyLightTier::Unknown
        }
    }

    /// Border AND. Unloaded neighbour = diverged. Edge columns only.
    fn border_sides_agree(
        level: &Level,
        chunk_pos: Vector2<i32>,
        local_x: i32,
        local_z: i32,
        height: SkyLightHeight,
    ) -> bool {
        let neighbors = [
            (
                local_x == 0,
                Vector2::new(chunk_pos.x - 1, chunk_pos.y),
                15,
                local_z,
            ),
            (
                local_x == 15,
                Vector2::new(chunk_pos.x + 1, chunk_pos.y),
                0,
                local_z,
            ),
            (
                local_z == 0,
                Vector2::new(chunk_pos.x, chunk_pos.y - 1),
                local_x,
                15,
            ),
            (
                local_z == 15,
                Vector2::new(chunk_pos.x, chunk_pos.y + 1),
                local_x,
                0,
            ),
        ];

        for (on_edge, neighbor_pos, neighbor_x, neighbor_z) in neighbors {
            if !on_edge {
                continue;
            }
            let agrees = level
                .read_chunk_sync(&neighbor_pos, |neighbor| {
                    let neighbor_height = SkyLightHeightMigration::get(neighbor);
                    height.border_uses_limit(
                        neighbor_height,
                        local_x,
                        local_z,
                        neighbor_x,
                        neighbor_z,
                    )
                })
                .unwrap_or(false);
            if !agrees {
                return false;
            }
        }
        true
    }

    /// Re-check this column; flag the quadrant if the ceiling left the band.
    fn refresh_sky_cut_after_change(cursor: &mut ChunkCursor, pos: &BlockPos) {
        let (chunk_pos, relative) = pos.chunk_and_chunk_relative_position();
        if let Some(chunk) = cursor.chunk_at(chunk_pos) {
            let cached = chunk.sky_light_height_cache.load(Ordering::Relaxed);
            if cached == 0 {
                return; // Never computed: the first compute will see this change anyway.
            }
            let height = SkyLightHeight::from_raw(cached);
            if !height.quadrant_uses_limit(relative.x, relative.z) {
                return; // Already diverged, nothing left to invalidate.
            }
            if !height.may_move_a_ceiling(
                pos.0.y,
                chunk.section.min_y,
                SkyLightHeight::chunk_height(chunk),
            ) {
                return;
            }

            let ceiling = SkyLightHeight::column_ceiling_at(chunk, relative.x, relative.z);
            if !height.ceiling_within_band(
                ceiling,
                chunk.section.min_y,
                SkyLightHeight::chunk_height(chunk),
            ) {
                SkyLightHeightMigration::mark_quadrant_diverged(chunk, relative.x, relative.z);
            }
        }
    }

    fn queues_empty(&self) -> bool {
        self.nodes_to_check.is_empty()
            && self.block_decrease.is_empty()
            && self.block_increase.is_empty()
            && self.sky_decrease.is_empty()
            && self.sky_increase.is_empty()
    }

    /// Vanilla `Level.setBlock` -> `LightEngine.checkBlock`: enqueue only, lock-free
    /// (see [`Self::propagate_lock`]). Flood is [`Self::drain_queued`].
    pub fn update_lighting_at(&self, _level: &Arc<Level>, pos: BlockPos) {
        self.nodes_to_check.push(pos);
    }

    /// Re-derives the light at every position queued since the last drain, each one once.
    fn check_pending_nodes(&self, cursor: &mut ChunkCursor) {
        let mut seen = FxHashSet::default();
        while let Some(pos) = self.nodes_to_check.pop() {
            if !seen.insert(pos) {
                continue;
            }
            // Block light needs its luminance, sky light its opacity, and nothing in between
            // changes the block. Fullbright and dark never look at it -> skip the fetch.
            let state = match cursor.level.lighting_config {
                LightingEngineConfig::Default => cursor.block_state(&pos),
                _ => pumpkin_data::Block::VOID_AIR.default_state,
            };
            self.check_block_light_updates_with_cursor(cursor, pos, state);
            // Must run before the sky pass: the pass reads the cut height this may invalidate.
            Self::refresh_sky_cut_after_change(cursor, &pos);
            self.check_sky_light_updates_with_cursor(cursor, pos, state);
        }
    }

    /// Vanilla `LightEngine.runLightUpdates`. One budgeted slice per tick so a sky
    /// refill into newly loaded chunks cannot dump the leftover onto the first `setBlock`.
    pub fn drain_queued(&self, level: &Arc<Level>) -> LightPassStats {
        let start = Instant::now();
        let mut updates = 0;
        if !self.queues_empty() {
            let _guard = self
                .propagate_lock
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let mut budget = LIGHT_UPDATES_PER_PASS;
            // One cursor for the whole pass: consecutive queue entries almost always
            // sit in the same chunk, so the hit rate climbs well beyond what a single
            // operation on its own could reach.
            // Dropped at the end of this block, so the tally has folded itself into the
            // shared counters before they are snapshotted below.
            let tally = LocalCounters::new(&self.counters);
            let mut cursor = ChunkCursor::new(level, &tally);
            self.check_pending_nodes(&mut cursor);
            updates += self.perform_block_light_updates(&mut cursor, &mut budget);
            updates += self.perform_sky_light_updates(&mut cursor, &mut budget);
        }
        let stats = LightPassStats::new(
            start.elapsed(),
            updates,
            !self.queues_empty(),
            self.counters.snapshot_and_reset(),
        );
        if stats.should_log() {
            debug!("light {stats}");
        }
        stats
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

    /// Runs `visit` for the six neighbours of `pos` that sit in a loaded chunk.
    ///
    /// Offset, resolve and skip stood in all four propagation loops. Vanilla treats a
    /// missing chunk as `Blocks.BEDROCK` (opaque); skipping it here means a write that
    /// cannot land never re-queues, stay bright or dark until the neighbour loads.
    ///
    /// `counter` is bumped per neighbour before the resolve, so the two light kinds keep
    /// counting under the names they always used.
    fn for_each_neighbor(
        cursor: &mut ChunkCursor,
        pos: &BlockPos,
        counter: Counter,
        mut visit: impl FnMut(&ChunkData, VerticalInChunk, BlockPos, BlockDirection),
    ) {
        for dir in BlockDirection::all() {
            let neighbor_pos = pos.offset(dir.to_offset());
            cursor.counters.bump(counter);
            let Some((chunk, cell)) = cursor.resolve(&neighbor_pos) else {
                continue;
            };
            visit(chunk, cell, neighbor_pos, dir);
        }
    }

    /// Drains one queue until it runs dry or the budget is spent, and reports how many
    /// entries it processed.
    ///
    /// The four `perform_*_{in,de}crease_updates` differed only in the queue, the counter
    /// and the propagation they drove. The budget and counting bookkeeping was the same in
    /// all of them and lives here now.
    fn drain_queue(
        &self,
        queue: &SegQueue<(BlockPos, u8)>,
        counter: Counter,
        cursor: &mut ChunkCursor,
        budget: &mut i32,
        propagate: fn(&Self, &mut ChunkCursor, &BlockPos, u8),
    ) -> i32 {
        let mut updates = 0;
        while *budget > 0 {
            let Some((pos, expected_light)) = queue.pop() else {
                break;
            };
            *budget -= 1;
            cursor.counters.bump(counter);
            propagate(self, cursor, &pos, expected_light);
            updates += 1;
        }
        updates
    }

    /// Alternates the decrease and the increase queue until neither moves any more.
    fn perform_block_light_updates(&self, cursor: &mut ChunkCursor, budget: &mut i32) -> i32 {
        let mut updates = 0;
        while *budget > 0 {
            let decreased = self.drain_queue(
                &self.block_decrease,
                Counter::BlockDecrease,
                cursor,
                budget,
                Self::propagate_block_light_decrease,
            );
            let increased = self.drain_queue(
                &self.block_increase,
                Counter::BlockIncrease,
                cursor,
                budget,
                Self::propagate_block_light_increase,
            );
            updates += decreased + increased;
            if decreased == 0 && increased == 0 {
                break;
            }
        }
        updates
    }

    fn propagate_block_light_increase(
        &self,
        cursor: &mut ChunkCursor,
        pos: &BlockPos,
        light_level: u8,
    ) {
        let from_id = cursor
            .resolve(pos)
            .map_or(BlockStateId::AIR, |(chunk, cell)| {
                ChunkCursor::state_id_at(chunk, cell)
            });
        let counters = cursor.counters;
        Self::for_each_neighbor(
            cursor,
            pos,
            Counter::GetBlockLight,
            |chunk, cell, neighbor_pos, dir| {
                counters.bump(Counter::BlockState);
                if let Some(new_light) =
                    ChunkCursor::raise_light(chunk, cell, from_id, dir, light_level, true)
                {
                    counters.bump(Counter::SetBlockLight);
                    if new_light > 1 {
                        self.queue_block_light_increase(neighbor_pos, new_light);
                    }
                }
            },
        );
    }

    fn propagate_block_light_decrease(
        &self,
        cursor: &mut ChunkCursor,
        pos: &BlockPos,
        removed_light_level: u8,
    ) {
        // Check what the current light level actually is at this position
        let current_level = cursor.block_light(pos).unwrap_or(0);

        // Only propagate decrease if this position hasn't already been reset to 0
        // This prevents positions that were intentionally set to 0 from propagating light
        if current_level == 0 && removed_light_level > 0 {
            // This position was already darkened, so propagate the darkness to neighbors
            let counters = cursor.counters;
            Self::for_each_neighbor(
                cursor,
                pos,
                Counter::GetBlockLight,
                |chunk, cell, neighbor_pos, dir| {
                    let Some(neighbor_light) = ChunkCursor::block_light_at(chunk, cell) else {
                        return;
                    };
                    if neighbor_light == 0 {
                        return;
                    }

                    counters.bump(Counter::BlockState);
                    let neighbor_state = ChunkCursor::block_state_at(chunk, cell);
                    if occlusion::face_occludes(neighbor_state.id, dir.opposite()) {
                        return;
                    }
                    let expected_from_removed_source =
                        decayed(removed_light_level, neighbor_state.opacity);

                    if neighbor_light <= expected_from_removed_source {
                        let neighbor_luminance = neighbor_state.luminance;
                        counters.bump(Counter::SetBlockLight);

                        if neighbor_luminance == 0 {
                            // No self-emission, darken it completely and continue propagation
                            ChunkCursor::write_light_at(chunk, cell, 0, true);
                            self.queue_block_light_decrease(neighbor_pos, neighbor_light);
                        } else {
                            // Has self-emission, set to its own light and re-propagate from it
                            ChunkCursor::write_light_at(chunk, cell, neighbor_luminance, true);
                            self.queue_block_light_increase(neighbor_pos, neighbor_luminance);
                        }
                    } else {
                        // This neighbor has brighter light from another source, re-propagate from it
                        self.queue_block_light_increase(neighbor_pos, neighbor_light);
                    }
                },
            );
        }
    }

    fn check_block_light_updates_with_cursor(
        &self,
        cursor: &mut ChunkCursor,
        pos: BlockPos,
        state: &'static pumpkin_data::BlockState,
    ) {
        cursor.counters.bump(Counter::CheckBlock);
        match cursor.level.lighting_config {
            // Pumpkin config, not vanilla: whole world fullbright / pitch black.
            LightingEngineConfig::Full => {
                cursor.set_block_light(&pos, 15);
                return;
            }
            LightingEngineConfig::Dark => {
                cursor.set_block_light(&pos, 0);
                return;
            }
            LightingEngineConfig::Default => {}
        }

        // An unloaded chunk keeps the previous behaviour on purpose: it reads as void air,
        // so nothing is written and the neighbour pass below still runs.
        cursor.counters.bump(Counter::GetBlockLight);
        let current_light = cursor
            .resolve(&pos)
            .and_then(|(chunk, cell)| ChunkCursor::block_light_at(chunk, cell))
            .unwrap_or(0);
        let expected_light = state.luminance;

        // Handle light decrease (removing light source or placing opaque block)
        if expected_light < current_light {
            // Set to expected value immediately, then queue decrease to darken neighbors
            cursor.set_block_light(&pos, expected_light);
            self.queue_block_light_decrease(pos, current_light);
        } else if expected_light > current_light {
            // Handle light increase (placing light source)
            cursor.set_block_light(&pos, expected_light);
            self.queue_block_light_increase(pos, expected_light);
        }

        // Only check neighbors if we didn't trigger a decrease
        // Decrease propagation handles re-validating neighbors
        if expected_light >= current_light {
            self.check_neighbors_light_updates_with_cursor(cursor, pos, expected_light);
        }
    }

    fn check_neighbors_light_updates_with_cursor(
        &self,
        cursor: &mut ChunkCursor,
        pos: BlockPos,
        current_light: u8,
    ) {
        for dir in BlockDirection::all() {
            let neighbor_pos = pos.offset(dir.to_offset());
            if let Some(neighbor_light) = cursor.block_light(&neighbor_pos)
                && neighbor_light > current_light + 1
            {
                self.queue_block_light_increase(neighbor_pos, neighbor_light);
            }
        }
    }

    /// Alternates the decrease and the increase queue until neither moves any more.
    fn perform_sky_light_updates(&self, cursor: &mut ChunkCursor, budget: &mut i32) -> i32 {
        let mut updates = 0;
        while *budget > 0 {
            let decreased = self.drain_queue(
                &self.sky_decrease,
                Counter::SkyDecrease,
                cursor,
                budget,
                Self::propagate_sky_light_decrease,
            );
            let increased = self.drain_queue(
                &self.sky_increase,
                Counter::SkyIncrease,
                cursor,
                budget,
                Self::propagate_sky_light_increase,
            );
            updates += decreased + increased;
            if decreased == 0 && increased == 0 {
                break;
            }
        }
        updates
    }

    fn propagate_sky_light_increase(
        &self,
        cursor: &mut ChunkCursor,
        pos: &BlockPos,
        light_level: u8,
    ) {
        let from_id = cursor
            .resolve(pos)
            .map_or(BlockStateId::AIR, |(chunk, cell)| {
                ChunkCursor::state_id_at(chunk, cell)
            });
        let counters = cursor.counters;
        Self::for_each_neighbor(
            cursor,
            pos,
            Counter::ChunkLoaded,
            |chunk, cell, neighbor_pos, dir| {
                counters.bump(Counter::GetSky);
                counters.bump(Counter::BlockState);
                if let Some(new_light) =
                    ChunkCursor::raise_light(chunk, cell, from_id, dir, light_level, false)
                {
                    counters.bump(Counter::SetSky);
                    if new_light > 0 {
                        self.queue_sky_light_increase(neighbor_pos, new_light);
                    }
                }
            },
        );
    }

    fn propagate_sky_light_decrease(
        &self,
        cursor: &mut ChunkCursor,
        pos: &BlockPos,
        removed_light: u8,
    ) {
        let counters = cursor.counters;
        Self::for_each_neighbor(
            cursor,
            pos,
            Counter::ChunkLoaded,
            |chunk, cell, neighbor_pos, dir| {
                counters.bump(Counter::GetSky);
                let neighbor_light = ChunkCursor::sky_light_at(chunk, cell);
                if neighbor_light == 0 {
                    return; // Already dark
                }

                counters.bump(Counter::BlockState);
                let state_id = ChunkCursor::state_id_at(chunk, cell);
                if occlusion::face_occludes(state_id, dir.opposite()) {
                    return;
                }
                let opacity = crate::lighting::opacity_of(state_id);

                // What the removed source would have given this neighbour
                let expected = if removed_light == 15 && dir == BlockDirection::Down && opacity == 0
                {
                    15
                } else {
                    decayed(removed_light, opacity)
                };

                if neighbor_light == expected || neighbor_light < removed_light {
                    // This neighbor was lit, darken it. Skip if the write
                    // cannot land (below `min_y` used to stay at sky=15 and loop).
                    counters.bump(Counter::SetSky);
                    if ChunkCursor::write_light_at(chunk, cell, 0, false) {
                        self.queue_sky_light_decrease(neighbor_pos, neighbor_light);
                    }
                } else if neighbor_light > removed_light {
                    // Neighbor has brighter light from another source
                    // Re-propagate from it to fill in the gap we left
                    self.queue_sky_light_increase(neighbor_pos, neighbor_light);
                }
            },
        );
    }

    fn check_sky_light_updates_with_cursor(
        &self,
        cursor: &mut ChunkCursor,
        pos: BlockPos,
        state: &'static pumpkin_data::BlockState,
    ) {
        cursor.counters.bump(Counter::CheckSky);
        match cursor.level.lighting_config {
            LightingEngineConfig::Full => {
                cursor.set_sky_light(&pos, 15);
                return;
            }
            LightingEngineConfig::Dark => {
                cursor.set_sky_light(&pos, 0);
                return;
            }
            LightingEngineConfig::Default => {}
        }

        // An unloaded chunk keeps the previous behaviour on purpose: dark, and void air
        // for the opacity.
        cursor.counters.bump(Counter::GetSky);
        let current_light = cursor
            .resolve(&pos)
            .map_or(0, |(chunk, cell)| ChunkCursor::sky_light_at(chunk, cell));
        let opacity = state.opacity;

        // Calculate expected sky light
        let expected_light = if opacity == 15 || state.is_solid_render() {
            // Fully opaque block = no light
            0
        } else {
            // Check if there's open sky above, cheaply where the cut height can decide it
            let has_sky = match Self::sky_tier(cursor, &pos) {
                SkyLightTier::NoOpenSky => {
                    cursor.counters.bump(Counter::SkyTier1);
                    false
                }
                SkyLightTier::OpenSky => {
                    cursor.counters.bump(Counter::SkyTier2);
                    true
                }
                SkyLightTier::Unknown => {
                    cursor.counters.bump(Counter::SkyTier3);
                    Self::has_open_sky_above(cursor, &pos)
                }
            };

            if has_sky {
                // Direct sunlight, reduced by opacity
                15u8.saturating_sub(opacity)
            } else {
                // No direct sky, take the brightest neighbour
                let mut best_light = 0;

                for dir in BlockDirection::all() {
                    let neighbor_light = cursor.sky_light(&pos.offset(dir.to_offset()));
                    // Sky light at 15 from directly above stays 15 through transparent blocks
                    let potential =
                        if neighbor_light == 15 && dir == BlockDirection::Up && opacity == 0 {
                            15
                        } else {
                            decayed(neighbor_light, opacity)
                        };

                    best_light = best_light.max(potential);
                    if best_light == 15 {
                        break;
                    }
                }

                best_light
            }
        };

        // Update if needed
        if expected_light < current_light {
            // Light decreased
            cursor.set_sky_light(&pos, expected_light);
            self.queue_sky_light_decrease(pos, current_light);
        } else if expected_light > current_light {
            // Light increased
            cursor.set_sky_light(&pos, expected_light);
            self.queue_sky_light_increase(pos, expected_light);
        }

        if expected_light >= current_light && expected_light > 0 {
            self.queue_sky_light_increase(pos, expected_light);
        }
    }

    // Public API for querying light levels. These methods are synchronous and may block if the
    // chunk is not loaded.

    pub fn get_block_light_level(&self, level: &Arc<Level>, position: &BlockPos) -> Option<u8> {
        ChunkCursor::new(level, &LocalCounters::new(&self.counters)).block_light(position)
    }

    pub fn get_sky_light_level(&self, level: &Arc<Level>, position: &BlockPos) -> u8 {
        ChunkCursor::new(level, &LocalCounters::new(&self.counters)).sky_light(position)
    }

    /// `Err` if the write cannot land (chunk not loaded or Y outside the
    /// chunk height).
    pub fn set_block_light_level(
        &self,
        level: &Arc<Level>,
        position: &BlockPos,
        light_level: u8,
    ) -> Result<(), String> {
        if ChunkCursor::new(level, &LocalCounters::new(&self.counters))
            .set_block_light(position, light_level)
        {
            Ok(())
        } else {
            Err("chunk not loaded or Y outside chunk height".to_string())
        }
    }

    pub fn set_sky_light_level(
        &self,
        level: &Arc<Level>,
        position: &BlockPos,
        light_level: u8,
    ) -> Result<(), String> {
        if ChunkCursor::new(level, &LocalCounters::new(&self.counters))
            .set_sky_light(position, light_level)
        {
            Ok(())
        } else {
            Err("chunk not loaded or Y outside chunk height".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ChunkCursor, ChunkData, DynamicLightEngine, LocalCounters, SkyLightHeightMigration,
        SkyLightTier,
    };
    use crate::chunk::format::LightContainer;
    use crate::level::Level;
    use pumpkin_config::world::LevelConfig;
    use pumpkin_data::Block;
    use pumpkin_data::dimension::Dimension;
    use pumpkin_util::math::position::BlockPos;
    use pumpkin_util::math::vector2::Vector2;
    use std::sync::Arc;
    use tempfile::TempDir;

    const SURFACE: i32 = 60;

    fn flat_chunk(cx: i32, cz: i32) -> Arc<ChunkData> {
        let chunk = ChunkData::empty(cx, cz);
        let mut updates = Vec::new();
        for x in 0..16usize {
            for z in 0..16usize {
                for y in 0..=SURFACE {
                    updates.push((x, y, z, Block::STONE.default_state.id));
                }
            }
        }
        chunk.set_blocks_batch(updates);
        *chunk
            .heightmap
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = chunk.calculate_heightmap();

        // `ChunkData::empty` starts with zero-length light storage, where every sky read
        // answers 15. A loaded chunk has one container per section.
        let mut light = chunk
            .light_engine
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        light.sky_light = (0..chunk.section.count)
            .map(|_| LightContainer::new_empty(0))
            .collect();
        light.block_light = (0..chunk.section.count)
            .map(|_| LightContainer::new_empty(0))
            .collect();
        drop(light);

        Arc::new(chunk)
    }

    fn level_with(positions: &[(i32, i32)]) -> (Arc<Level>, TempDir) {
        let dir = TempDir::new().expect("temp dir");
        let level = Level::from_root_folder(
            &LevelConfig::default(),
            dir.path().to_path_buf(),
            42,
            Dimension::OVERWORLD,
        );
        for &(cx, cz) in positions {
            level
                .loaded_chunks
                .insert(Vector2::new(cx, cz), flat_chunk(cx, cz));
        }
        (level, dir)
    }

    /// At a chunk border the fast tier answer holds only if the neighbour's near-border
    /// quadrant carries it too, and a neighbour that is not loaded counts as diverged.
    /// checked the wiring around AND: that [`DynamicLightEngine::sky_tier`] consults the
    /// neighbour at all, that it picks the right one of the four sides, and that it leaves
    /// inland columns alone -> don't pay for a border they are not on.
    #[tokio::test]
    async fn the_border_gate_downgrades_only_edge_columns() {
        // Deep below the cut, where the chunk-local answer is a fast one.
        let border = BlockPos::new(15, 20, 2);
        let inland = BlockPos::new(8, 20, 2);
        let engine = DynamicLightEngine::new();

        let (level, _dir) = level_with(&[(0, 0), (1, 0)]);
        let tally = LocalCounters::new(&engine.counters);
        let mut cursor = ChunkCursor::new(&level, &tally);
        assert_eq!(
            DynamicLightEngine::sky_tier(&mut cursor, &border),
            SkyLightTier::NoOpenSky,
            "two untouched chunks: the border column keeps the fast path"
        );

        let neighbour = level
            .loaded_chunks
            .get(&Vector2::new(1, 0))
            .expect("the neighbour was loaded")
            .value()
            .clone();
        SkyLightHeightMigration::get(&neighbour);
        SkyLightHeightMigration::mark_quadrant_diverged(&neighbour, 0, 2);

        assert_eq!(
            DynamicLightEngine::sky_tier(&mut cursor, &border),
            SkyLightTier::Unknown,
            "the adjoining quadrant across the border diverged, so the fast answer no \
             longer holds for this column"
        );
        assert_eq!(
            DynamicLightEngine::sky_tier(&mut cursor, &inland),
            SkyLightTier::NoOpenSky,
            "a column that is not on the border must not pay for the neighbour"
        );

        // The same column again, with nothing at all on the other side.
        let (lonely, _lonely_dir) = level_with(&[(0, 0)]);
        let lonely_tally = LocalCounters::new(&engine.counters);
        let mut cursor = ChunkCursor::new(&lonely, &lonely_tally);
        assert_eq!(
            DynamicLightEngine::sky_tier(&mut cursor, &border),
            SkyLightTier::Unknown,
            "an unloaded neighbour has to count as diverged"
        );
        assert_eq!(
            DynamicLightEngine::sky_tier(&mut cursor, &inland),
            SkyLightTier::NoOpenSky
        );
    }

    /// Only a real light property may decide whether the engine runs. The premises are
    /// asserted, so the test fails loudly if the block data stops backing them.
    #[test]
    fn a_change_is_light_neutral_exactly_when_both_properties_match() {
        let stone = Block::STONE.default_state;
        let dirt = Block::DIRT.default_state;
        let air = Block::AIR.default_state;
        let glowstone = Block::GLOWSTONE.default_state;

        assert_eq!(stone.opacity, dirt.opacity, "premise: both fully opaque");
        assert_eq!(stone.luminance, dirt.luminance, "premise: neither glows");
        assert!(
            !crate::lighting::LightEngine::has_different_light_properties(stone, dirt),
            "swapping one opaque block for another cannot move any light"
        );

        assert_ne!(stone.opacity, air.opacity, "premise: opacity differs");
        assert!(
            crate::lighting::LightEngine::has_different_light_properties(stone, air),
            "opening a solid block up has to reach the engine"
        );

        assert_ne!(stone.luminance, glowstone.luminance, "premise: one glows");
        assert!(
            crate::lighting::LightEngine::has_different_light_properties(stone, glowstone),
            "a block that starts glowing has to reach the engine"
        );

        let stair_a = Block::OAK_STAIRS.default_state;
        let stair_b = Block::OAK_STAIRS
            .states
            .iter()
            .find(|s| s.id != stair_a.id)
            .expect("stairs have more than one state");
        assert!(
            crate::lighting::LightEngine::has_different_light_properties(stair_a, stair_b),
            "rotating a stair must reach the engine"
        );
    }

    /// Vanilla `blockNodesToCheck` collapses repeats.
    /// drain is checked once, against the state it ended on. Only the end state can be
    /// observed, because nothing drained in between.
    #[tokio::test]
    async fn repeated_touches_settle_on_the_state_the_position_ended_on() {
        let (level, _dir) = level_with(&[(0, 0)]);
        let pos = BlockPos::new(8, SURFACE + 3, 8);
        let chunk = level
            .loaded_chunks
            .get(&Vector2::new(0, 0))
            .expect("loaded")
            .clone();

        let settle = |engine: &DynamicLightEngine| {
            assert!(
                (0..64).any(|_| !engine.drain_queued(&level).leftover),
                "light updates did not converge"
            );
        };

        let set = |id| {
            chunk.set_block_absolute_y(8, pos.0.y, 8, id);
        };

        // Toggled several times, ending lit.
        let many = DynamicLightEngine::new();
        for i in 0..8 {
            set(if i % 2 == 0 {
                Block::GLOWSTONE.default_state.id
            } else {
                Block::AIR.default_state.id
            });
            many.update_lighting_at(&level, pos);
        }
        set(Block::GLOWSTONE.default_state.id);
        many.update_lighting_at(&level, pos);
        settle(&many);
        let after_many = many.get_block_light_level(&level, &pos);

        assert_eq!(
            after_many,
            Some(Block::GLOWSTONE.default_state.luminance),
            "the surviving glowstone must light its own cell"
        );

        // The same end state reached in one touch has to agree.
        let once = DynamicLightEngine::new();
        set(Block::AIR.default_state.id);
        once.update_lighting_at(&level, pos);
        settle(&once);
        set(Block::GLOWSTONE.default_state.id);
        once.update_lighting_at(&level, pos);
        settle(&once);

        assert_eq!(
            once.get_block_light_level(&level, &pos),
            after_many,
            "collapsing the repeats changed the result"
        );
    }
}
