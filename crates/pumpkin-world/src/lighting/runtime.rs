use crate::chunk::palette::BlockPalette;
use crate::level::Level;
use crate::lighting::chunk_access::ChunkCursor;
use crate::lighting::sky_light_height::{SkyLightHeight, SkyLightHeightMigration, SkyLightTier};
use crate::lighting::stats::{Counter, LightCounters, LightPassStats};
use crossbeam::queue::SegQueue;
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
    /// `ServerLevel` lighting is single-threaded. Rayon random ticks and the
    /// net thread only `checkBlock`; two `perform_*` loops ping-pong and
    /// never empty.
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
    fn has_open_sky_above(&self, cursor: &mut ChunkCursor, pos: &BlockPos) -> bool {
        self.counters.bump(Counter::SkyColumnScan);
        // The whole column is in the same chunk by definition, so it is resolved once and
        // the pointer then stays in a register for every step. Re-resolving per step would
        // repeat the position split and the memo compare up to ~250 times for one chunk.
        let Some(chunk) = cursor.chunk_for(pos) else {
            return false;
        };
        let min_y = chunk.section.min_y;
        let max_y = min_y + (chunk.section.count as i32) * BlockPalette::SIZE as i32 - 1;
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
                    .map(|section| {
                        section
                            .get(local_x, rel_y % BlockPalette::SIZE, local_z)
                            .to_state()
                    })
                    .map_or(0, |state| state.opacity);
                if opacity > 0 {
                    return (true, reads);
                }
            }
            (false, reads)
        });
        self.counters.bump_n(Counter::SkyColumnRead, reads);

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
        self.block_decrease.is_empty()
            && self.block_increase.is_empty()
            && self.sky_decrease.is_empty()
            && self.sky_increase.is_empty()
    }

    /// Vanilla `Level.setBlock` -> `LightEngine.checkBlock`: enqueue only.
    /// Flood is [`Self::drain_queued`]. Sync on the tick thread (not the light thread).
    pub fn update_lighting_at(&self, level: &Arc<Level>, pos: BlockPos) {
        let _guard = self
            .propagate_lock
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        // All three steps work on the same chunk; one cursor saves their lookups.
        let mut cursor = ChunkCursor::new(level, &self.counters);
        self.check_block_light_updates_with_cursor(&mut cursor, pos);
        // Must run before the sky pass: the pass reads the cut height this may invalidate.
        Self::refresh_sky_cut_after_change(&mut cursor, &pos);
        self.check_sky_light_updates_with_cursor(&mut cursor, pos);
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
            let mut cursor = ChunkCursor::new(level, &self.counters);
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

    fn perform_block_light_updates(&self, cursor: &mut ChunkCursor, budget: &mut i32) -> i32 {
        let mut updates = 0;

        loop {
            if *budget <= 0 {
                break;
            }
            let decrease_updates = self.perform_block_light_decrease_updates(cursor, budget);
            let increase_updates = self.perform_block_light_increase_updates(cursor, budget);

            updates += decrease_updates + increase_updates;

            if decrease_updates == 0 && increase_updates == 0 {
                break;
            }
        }

        updates
    }

    fn perform_block_light_decrease_updates(
        &self,
        cursor: &mut ChunkCursor,
        budget: &mut i32,
    ) -> i32 {
        let mut updates = 0;

        while *budget > 0 {
            let Some((pos, expected_light)) = self.block_decrease.pop() else {
                break;
            };
            *budget -= 1;
            self.counters.bump(Counter::BlockDecrease);
            self.propagate_block_light_decrease(cursor, &pos, expected_light);
            updates += 1;
        }

        updates
    }

    fn perform_block_light_increase_updates(
        &self,
        cursor: &mut ChunkCursor,
        budget: &mut i32,
    ) -> i32 {
        let mut updates = 0;

        while *budget > 0 {
            let Some((pos, expected_light)) = self.block_increase.pop() else {
                break;
            };
            *budget -= 1;
            self.counters.bump(Counter::BlockIncrease);
            self.propagate_block_light_increase(cursor, &pos, expected_light);
            updates += 1;
        }

        updates
    }

    fn propagate_block_light_increase(
        &self,
        cursor: &mut ChunkCursor,
        pos: &BlockPos,
        light_level: u8,
    ) {
        for dir in BlockDirection::all() {
            let neighbor_pos = pos.offset(dir.to_offset());

            let counters = cursor.counters;
            counters.bump(Counter::GetBlockLight);
            let Some(chunk) = cursor.chunk_for(&neighbor_pos) else {
                continue;
            };
            if let Some(neighbor_light) = ChunkCursor::block_light_in(chunk, &neighbor_pos) {
                counters.bump(Counter::BlockState);
                let opacity = ChunkCursor::opacity_in(chunk, &neighbor_pos).max(1);
                let new_light = light_level.saturating_sub(opacity);

                // Only propagate if new light is brighter than current light
                if new_light > neighbor_light {
                    counters.bump(Counter::SetBlockLight);
                    let written = ChunkCursor::write_light(chunk, &neighbor_pos, new_light, true);
                    if written && new_light > 1 {
                        self.queue_block_light_increase(neighbor_pos, new_light);
                    }
                }
            }
        }
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
            // This position was already darkened, so we propagate the darkness to neighbors
            for dir in BlockDirection::all() {
                let neighbor_pos = pos.offset(dir.to_offset());

                let counters = cursor.counters;
                counters.bump(Counter::GetBlockLight);
                let Some(chunk) = cursor.chunk_for(&neighbor_pos) else {
                    continue;
                };

                if let Some(neighbor_light) = ChunkCursor::block_light_in(chunk, &neighbor_pos) {
                    if neighbor_light == 0 {
                        continue; // Skip if already 0
                    }

                    counters.bump(Counter::BlockState);
                    let neighbor_state = ChunkCursor::block_state_in(chunk, &neighbor_pos);
                    let opacity = neighbor_state.opacity.max(1);

                    let expected_from_removed_source = removed_light_level.saturating_sub(opacity);

                    if neighbor_light <= expected_from_removed_source {
                        let neighbor_luminance = neighbor_state.luminance;
                        counters.bump(Counter::SetBlockLight);

                        if neighbor_luminance == 0 {
                            // No self-emission, darken it completely and continue propagation
                            ChunkCursor::write_light(chunk, &neighbor_pos, 0, true);
                            self.queue_block_light_decrease(neighbor_pos, neighbor_light);
                        } else {
                            // Has self-emission, set to its own light and re-propagate from it
                            ChunkCursor::write_light(
                                chunk,
                                &neighbor_pos,
                                neighbor_luminance,
                                true,
                            );
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
        let mut cursor = ChunkCursor::new(level, &self.counters);
        self.check_block_light_updates_with_cursor(&mut cursor, pos);
    }

    fn check_block_light_updates_with_cursor(&self, cursor: &mut ChunkCursor, pos: BlockPos) {
        self.counters.bump(Counter::CheckBlock);
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

        // Both reads of `pos` share one resolve. An unloaded chunk keeps the previous
        // behaviour on purpose: it reads as void air, so nothing is written and the
        // neighbour pass below still runs.
        let counters = cursor.counters;
        counters.bump(Counter::GetBlockLight);
        counters.bump(Counter::BlockState);
        let (current_light, expected_light) = cursor.chunk_for(&pos).map_or_else(
            || (0, pumpkin_data::Block::VOID_AIR.default_state.luminance),
            |chunk| {
                (
                    ChunkCursor::block_light_in(chunk, &pos).unwrap_or(0),
                    ChunkCursor::block_state_in(chunk, &pos).luminance,
                )
            },
        );

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
        let mut cursor = ChunkCursor::new(level, &self.counters);
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

    fn perform_sky_light_updates(&self, cursor: &mut ChunkCursor, budget: &mut i32) -> i32 {
        let mut updates = 0;
        loop {
            if *budget <= 0 {
                break;
            }
            let decrease_updates = self.perform_sky_light_decrease_updates(cursor, budget);
            let increase_updates = self.perform_sky_light_increase_updates(cursor, budget);

            updates += decrease_updates + increase_updates;

            if decrease_updates == 0 && increase_updates == 0 {
                break;
            }
        }
        updates
    }

    fn perform_sky_light_decrease_updates(
        &self,
        cursor: &mut ChunkCursor,
        budget: &mut i32,
    ) -> i32 {
        let mut updates = 0;
        while *budget > 0 {
            let Some((pos, expected_light)) = self.sky_decrease.pop() else {
                break;
            };
            *budget -= 1;
            self.counters.bump(Counter::SkyDecrease);
            self.propagate_sky_light_decrease(cursor, &pos, expected_light);
            updates += 1;
        }
        updates
    }

    fn perform_sky_light_increase_updates(
        &self,
        cursor: &mut ChunkCursor,
        budget: &mut i32,
    ) -> i32 {
        let mut updates = 0;
        while *budget > 0 {
            let Some((pos, expected_light)) = self.sky_increase.pop() else {
                break;
            };
            *budget -= 1;
            self.counters.bump(Counter::SkyIncrease);
            self.propagate_sky_light_increase(cursor, &pos, expected_light);
            updates += 1;
        }
        updates
    }

    fn propagate_sky_light_increase(
        &self,
        cursor: &mut ChunkCursor,
        pos: &BlockPos,
        light_level: u8,
    ) {
        for dir in BlockDirection::all() {
            let neighbor_pos = pos.offset(dir.to_offset());

            // Vanilla missing chunk is `Blocks.BEDROCK` (opaque). Skip here so a
            // dropped write cannot re-queue forever; the seam can stay bright/dark
            // until the neighbour loads.
            //
            // Resolved once and reused for the read, the opacity and the write below.
            let counters = cursor.counters;
            counters.bump(Counter::ChunkLoaded);
            let Some(chunk) = cursor.chunk_for(&neighbor_pos) else {
                continue;
            };

            counters.bump(Counter::GetSky);
            let neighbor_light = ChunkCursor::sky_light_in(chunk, &neighbor_pos);
            counters.bump(Counter::BlockState);
            let opacity = ChunkCursor::opacity_in(chunk, &neighbor_pos);

            // Calculate new light level for neighbor
            let new_light = if light_level == 15 && dir == BlockDirection::Down && opacity == 0 {
                // Special case: Sky light at 15 propagates down as 15 through transparent blocks
                15
            } else {
                // Normal propagation: reduce by 1 for distance, then by opacity
                light_level.saturating_sub(1).saturating_sub(opacity)
            };

            // Only propagate if new light is brighter than current light.
            // `set` fails outside the chunk height; do not re-queue those.
            if new_light > neighbor_light {
                counters.bump(Counter::SetSky);
                let written = ChunkCursor::write_light(chunk, &neighbor_pos, new_light, false);
                if written && new_light > 0 {
                    self.queue_sky_light_increase(neighbor_pos, new_light);
                }
            }
        }
    }

    fn propagate_sky_light_decrease(
        &self,
        cursor: &mut ChunkCursor,
        pos: &BlockPos,
        removed_light: u8,
    ) {
        for dir in BlockDirection::all() {
            let neighbor_pos = pos.offset(dir.to_offset());

            let counters = cursor.counters;
            counters.bump(Counter::ChunkLoaded);
            let Some(chunk) = cursor.chunk_for(&neighbor_pos) else {
                continue;
            };

            counters.bump(Counter::GetSky);
            let neighbor_light = ChunkCursor::sky_light_in(chunk, &neighbor_pos);
            if neighbor_light == 0 {
                continue; // Already dark
            }

            counters.bump(Counter::BlockState);
            let opacity = ChunkCursor::opacity_in(chunk, &neighbor_pos);

            // Calculate what we would have given this neighbor
            let expected = if removed_light == 15 && dir == BlockDirection::Down && opacity == 0 {
                15
            } else {
                removed_light.saturating_sub(1).saturating_sub(opacity)
            };

            if neighbor_light == expected || neighbor_light < removed_light {
                // This neighbor was lit by us, darken it. Skip if the write
                // cannot land (below `min_y` used to stay at sky=15 and loop).
                counters.bump(Counter::SetSky);
                if ChunkCursor::write_light(chunk, &neighbor_pos, 0, false) {
                    self.queue_sky_light_decrease(neighbor_pos, neighbor_light);
                }
            } else if neighbor_light > removed_light {
                // Neighbor has brighter light from another source
                // Re-propagate from it to fill in the gap we left
                self.queue_sky_light_increase(neighbor_pos, neighbor_light);
            }
        }
    }

    pub fn check_sky_light_updates(&self, level: &Arc<Level>, pos: BlockPos) {
        let mut cursor = ChunkCursor::new(level, &self.counters);
        self.check_sky_light_updates_with_cursor(&mut cursor, pos);
    }

    fn check_sky_light_updates_with_cursor(&self, cursor: &mut ChunkCursor, pos: BlockPos) {
        self.counters.bump(Counter::CheckSky);
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

        // Both reads of `pos` share one resolve. An unloaded chunk keeps the previous
        // behaviour on purpose: dark, and void air for the opacity.
        let counters = cursor.counters;
        counters.bump(Counter::GetSky);
        counters.bump(Counter::BlockState);
        let (current_light, opacity) = cursor.chunk_for(&pos).map_or_else(
            || (0, pumpkin_data::Block::VOID_AIR.default_state.opacity),
            |chunk| {
                (
                    ChunkCursor::sky_light_in(chunk, &pos),
                    ChunkCursor::opacity_in(chunk, &pos),
                )
            },
        );

        // Calculate expected sky light
        let expected_light = if opacity == 15 {
            // Fully opaque block = no light
            0
        } else {
            // Check if there's open sky above, cheaply where the cut height can decide it
            let has_sky = match Self::sky_tier(cursor, &pos) {
                SkyLightTier::NoOpenSky => {
                    self.counters.bump(Counter::SkyTier1);
                    false
                }
                SkyLightTier::OpenSky => {
                    self.counters.bump(Counter::SkyTier2);
                    true
                }
                SkyLightTier::Unknown => {
                    self.counters.bump(Counter::SkyTier3);
                    self.has_open_sky_above(cursor, &pos)
                }
            };

            if has_sky {
                // Direct sunlight, reduced by opacity
                15u8.saturating_sub(opacity)
            } else {
                // No direct sky, check neighbors for best light
                let mut best_light = 0;

                for dir in BlockDirection::all() {
                    let neighbor_pos = pos.offset(dir.to_offset());

                    let neighbor_light = cursor.sky_light(&neighbor_pos);
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
        ChunkCursor::new(level, &self.counters).block_light(position)
    }

    pub fn get_sky_light_level_sync(&self, level: &Level, position: &BlockPos) -> u8 {
        ChunkCursor::new(level, &self.counters).sky_light(position)
    }

    pub fn get_block_light_level(&self, level: &Arc<Level>, position: &BlockPos) -> Option<u8> {
        ChunkCursor::new(level, &self.counters).block_light(position)
    }

    pub fn get_sky_light_level(&self, level: &Arc<Level>, position: &BlockPos) -> u8 {
        ChunkCursor::new(level, &self.counters).sky_light(position)
    }

    /// `Err` if the write cannot land (chunk not loaded or Y outside the
    /// chunk height).
    pub fn set_block_light_level(
        &self,
        level: &Arc<Level>,
        position: &BlockPos,
        light_level: u8,
    ) -> Result<(), String> {
        if ChunkCursor::new(level, &self.counters).set_block_light(position, light_level) {
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
        if ChunkCursor::new(level, &self.counters).set_sky_light(position, light_level) {
            Ok(())
        } else {
            Err("chunk not loaded or Y outside chunk height".to_string())
        }
    }
}
