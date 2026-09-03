use crate::chunk::ChunkData;
use crate::chunk::palette::BlockPalette;
use crate::level::Level;
use crate::lighting::chunk_access::{ChunkCursor, VerticalInChunk};
use crate::lighting::decayed;
use crate::lighting::sky_light_height::{SkyLightHeight, SkyLightHeightMigration, SkyLightTier};
use crate::lighting::stats::{Counter, LightCounters, LightPassStats, LocalCounters};
use crossbeam::queue::SegQueue;
use rustc_hash::FxHashSet;
use pumpkin_config::lighting::LightingEngineConfig;
use pumpkin_data::BlockDirection;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector2::Vector2;
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
    /// Open sky above `pos` in this column. Vanilla `SkyLightEngine` uses sky-section
    /// sources, not a per-`checkBlock` scan; overhangs can disagree until the flood catches up.
    /// Bounded by this chunk's height (`VOID_AIR` opacity 0 would walk forever past `max_y`).
    fn has_open_sky_above(cursor: &mut ChunkCursor, pos: &BlockPos) -> bool {
        cursor.counters.bump(Counter::SkyColumnScan);
        // The whole column is in the same chunk by definition, so it is resolved once and
        // the pointer then stays in a register for every step. Re-resolving per step would
        // repeat the position split and the memo compare up to ~250 times for one chunk.
        let Some(chunk) = cursor.chunk_for(pos) else {
            return false;
        };
        let min_y = chunk.section.min_y;
        let max_y = min_y + SkyLightHeight::chunk_height(chunk) - 1;
        let (_, relative) = pos.chunk_and_chunk_relative_position();
        let (local_x, local_z) = (relative.x as usize, relative.z as usize);

        // The sections read guard is taken once for the whole column instead of once per
        // block. It protects the same data either way; per-step locking is a few hundred
        // uncontended round trips that buy nothing.
        let (blocked, reads) = chunk.section.with_blocks(|sections| {
            let mut reads = 0u64;
            for y in (pos.0.y + 1)..=max_y {
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
            return tier; // Unclear even without a border: saves the neighbour lookup.
        }
        // Deliberately not via the cursor: the neighbour chunk would evict its memo,
        // even though the caller carries on in its own chunk right after. Only the
        // edge column pays this, and there it is 1-2 lookups.
        if Self::border_sides_agree(cursor.level, chunk_pos, relative.x, relative.z, height) {
            tier
        } else {
            SkyLightTier::Unknown
        }
    }

    /// Chunk border sync: at a chunk border the neighbour's near-border quadrant has to
    /// carry the fast path too (AND). If it does not, or the neighbour is not
    /// loaded, the position falls back to the real check (NAND).
    ///
    /// Only the edge column pays (`local == 0 || local == 15`), a corner pays two sides.
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

    /// Keeps the cached cut height honest after a block change.
    ///
    /// A non-diverged quadrant promises every column ceiling sits in
    /// `[cut, cut + spread]`. Only a change at or above the cut can
    /// break that: digging below the cut leaves the occluders above it untouched, and
    /// placing below the cut cannot raise a ceiling that is already at or above it. So we
    /// re-derive this one column and flag the quadrant if it left the band.
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
            // Both bounds live in `SkyLightHeight` so they share one rounding tolerance
            // with `tier()`; a local copy of either is how the band drifts apart.
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

    /// Whether a block change can move any light at all.
    ///
    /// Sky light reads `opacity`, block light `luminance` and `opacity`
    #[must_use]
    pub const fn block_change_affects_light(
        old: &pumpkin_data::BlockState,
        new: &pumpkin_data::BlockState,
    ) -> bool {
        old.opacity != new.opacity || old.luminance != new.luminance
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
        // A shared reference, so copying it out keeps no borrow of the cursor that the
        // walk below needs mutably.
        let counters = cursor.counters;
        Self::for_each_neighbor(
            cursor,
            pos,
            Counter::GetBlockLight,
            |chunk, cell, neighbor_pos, _dir| {
                let Some(neighbor_light) = ChunkCursor::block_light_at(chunk, cell) else {
                    return;
                };
                counters.bump(Counter::BlockState);
                let new_light = decayed(light_level, ChunkCursor::opacity_at(chunk, cell));

                // Only propagate if new light is brighter than current light
                if new_light > neighbor_light {
                    counters.bump(Counter::SetBlockLight);
                    let written = ChunkCursor::write_light_at(chunk, cell, new_light, true);
                    if written && new_light > 1 {
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
                |chunk, cell, neighbor_pos, _dir| {
                    let Some(neighbor_light) = ChunkCursor::block_light_at(chunk, cell) else {
                        return;
                    };
                    if neighbor_light == 0 {
                        return; // Skip if already 0
                    }

                    counters.bump(Counter::BlockState);
                    let neighbor_state = ChunkCursor::block_state_at(chunk, cell);
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

    pub fn check_block_light_updates(&self, level: &Arc<Level>, pos: BlockPos) {
        let tally = LocalCounters::new(&self.counters);
        let mut cursor = ChunkCursor::new(level, &tally);
        let state = cursor.block_state(&pos);
        self.check_block_light_updates_with_cursor(&mut cursor, pos, state);
    }

    /// `state` is the block state at `pos`, passed in so that
    /// [`Self::update_lighting_at`] can share one fetch with the sky pass.
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

    pub fn check_neighbors_light_updates(
        &self,
        level: &Arc<Level>,
        pos: BlockPos,
        current_light: u8,
    ) {
        let tally = LocalCounters::new(&self.counters);
        let mut cursor = ChunkCursor::new(level, &tally);
        self.check_neighbors_light_updates_with_cursor(&mut cursor, pos, current_light);
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
        let counters = cursor.counters;
        Self::for_each_neighbor(
            cursor,
            pos,
            Counter::ChunkLoaded,
            |chunk, cell, neighbor_pos, dir| {
                counters.bump(Counter::GetSky);
                let neighbor_light = ChunkCursor::sky_light_at(chunk, cell);
                counters.bump(Counter::BlockState);
                let opacity = ChunkCursor::opacity_at(chunk, cell);

                // Sky light at 15 propagates down as 15 through transparent blocks
                let new_light = if light_level == 15 && dir == BlockDirection::Down && opacity == 0
                {
                    15
                } else {
                    decayed(light_level, opacity)
                };

                // Only propagate if new light is brighter than current light.
                // `set` fails outside the chunk height; do not re-queue those.
                if new_light > neighbor_light {
                    counters.bump(Counter::SetSky);
                    let written = ChunkCursor::write_light_at(chunk, cell, new_light, false);
                    if written && new_light > 0 {
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
                let opacity = ChunkCursor::opacity_at(chunk, cell);

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

    pub fn check_sky_light_updates(&self, level: &Arc<Level>, pos: BlockPos) {
        let tally = LocalCounters::new(&self.counters);
        let mut cursor = ChunkCursor::new(level, &tally);
        let state = cursor.block_state(&pos);
        self.check_sky_light_updates_with_cursor(&mut cursor, pos, state);
    }

    /// `state` is the block state at `pos`; see
    /// [`Self::check_block_light_updates_with_cursor`].
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
        let expected_light = if opacity == 15 {
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

        // Keep spreading if light increased or stayed the same
        if expected_light >= current_light {
            self.queue_sky_light_spread(pos, expected_light);
        }
    }

    /// Re-queues `pos` so the flood continues outward from it.
    pub fn queue_sky_light_spread(&self, pos: BlockPos, current_light: u8) {
        if current_light > 0 {
            self.queue_sky_light_increase(pos, current_light);
        }
    }

    // Public API for querying light levels. These methods are synchronous and may block if the
    // chunk is not loaded.

    pub fn get_block_light_level_sync(&self, level: &Level, position: &BlockPos) -> Option<u8> {
        ChunkCursor::new(level, &LocalCounters::new(&self.counters)).block_light(position)
    }

    pub fn get_sky_light_level_sync(&self, level: &Level, position: &BlockPos) -> u8 {
        ChunkCursor::new(level, &LocalCounters::new(&self.counters)).sky_light(position)
    }

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
        ChunkCursor, ChunkData, Counter, DynamicLightEngine, LocalCounters,
        SkyLightHeightMigration, SkyLightTier,
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

    /// Only `opacity` and `luminance` may decide whether the engine runs. The premises
    /// are asserted
    #[test]
    fn a_change_is_light_neutral_exactly_when_both_properties_match() {
        let stone = Block::STONE.default_state;
        let dirt = Block::DIRT.default_state;
        let air = Block::AIR.default_state;
        let glowstone = Block::GLOWSTONE.default_state;

        assert_eq!(stone.opacity, dirt.opacity, "premise: both fully opaque");
        assert_eq!(stone.luminance, dirt.luminance, "premise: neither glows");
        assert!(
            !DynamicLightEngine::block_change_affects_light(stone, dirt),
            "swapping one opaque block for another cannot move any light"
        );

        assert_ne!(stone.opacity, air.opacity, "premise: opacity differs");
        assert!(
            DynamicLightEngine::block_change_affects_light(stone, air),
            "opening a solid block up has to reach the engine"
        );

        assert_ne!(stone.luminance, glowstone.luminance, "premise: one glows");
        assert!(
            DynamicLightEngine::block_change_affects_light(stone, glowstone),
            "a block that starts glowing has to reach the engine"
        );
    }

    /// The hot path counters
    ///
    /// `stats.rs` reads them as "six per propagated cell", and every judgement made from a
    /// light log rests on that.
    ///
    /// One propagation step in solid rock, which cannot cascade: the neighbours swallow the
    /// light, so exactly one queue entry is processed and exactly six neighbours are seen.
    #[tokio::test]
    async fn one_propagation_step_counts_six_neighbours() {
        let (level, _dir) = level_with(&[(0, 0)]);
        let engine = DynamicLightEngine::new();
        let buried = BlockPos::new(8, 20, 8);

        engine.queue_sky_light_increase(buried, 5);
        let sky = engine.drain_queued(&level);
        assert_eq!(sky.count(Counter::SkyIncrease), 1, "one queue entry");
        assert_eq!(
            sky.count(Counter::ChunkLoaded),
            6,
            "one chunk resolve per neighbour"
        );
        assert_eq!(
            sky.count(Counter::GetSky),
            6,
            "one light read per neighbour"
        );
        assert_eq!(
            sky.count(Counter::BlockState),
            6,
            "one opacity lookup per neighbour"
        );
        assert_eq!(
            sky.count(Counter::SetSky),
            0,
            "stone swallows the light, so nothing is written"
        );

        engine.queue_block_light_increase(buried, 5);
        let block = engine.drain_queued(&level);
        assert_eq!(block.count(Counter::BlockIncrease), 1);
        assert_eq!(
            block.count(Counter::GetBlockLight),
            6,
            "the block light loop walks the same six neighbours"
        );
        assert_eq!(block.count(Counter::BlockState), 6);
        assert_eq!(block.count(Counter::SetBlockLight), 0);

        // At the edge of the loaded area the two numbers part company: the resolve is
        // attempted for all six, and only the five that land are read. Counting after the
        // resolve instead would hide exactly the neighbours that cost a failed lookup.
        let at_edge = BlockPos::new(0, 20, 8);
        engine.queue_sky_light_increase(at_edge, 5);
        let edge = engine.drain_queued(&level);
        assert_eq!(
            edge.count(Counter::ChunkLoaded),
            6,
            "every neighbour is looked up, loaded or not"
        );
        assert_eq!(
            edge.count(Counter::GetSky),
            5,
            "the neighbour in the unloaded chunk is never read"
        );
        assert_eq!(edge.count(Counter::BlockState), 5);
    }
    /// Vanilla `blockNodesToCheck` collapses repeats: a position touched many times before a
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
