use crate::chunk::ChunkData;
use crate::chunk::io::Dirtiable;
use crate::chunk::palette::BlockPalette;
use crate::level::Level;
use crate::lighting::sky_light_height::{SkyLightHeight, SkyLightHeightMigration, SkyLightTier};
use crossbeam::queue::SegQueue;
use pumpkin_config::lighting::LightingEngineConfig;
use pumpkin_data::BlockDirection;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector2::Vector2;
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::debug;

/// One `drain_queued` slice per `ServerLevel.tick`. Vanilla `LightEngine.runLightUpdates`
/// empties the queues; `ThreadedLevelLightEngine` does that on the light thread.
/// Leftover is visible as delayed shadows after mining, placing, or a chunk-border sky refill.
const LIGHT_UPDATES_PER_PASS: i32 = 16_384;

const LIGHT_COUNTER_NAMES: [&str; 17] = [
    "check_block",
    "check_sky",
    "sky_column_scan",
    "sky_column_read",
    "sky_increase",
    "sky_decrease",
    "block_increase",
    "block_decrease",
    "get_sky",
    "set_sky",
    "get_block_light",
    "set_block_light",
    "block_state",
    "chunk_loaded",
    "sky_tier1_no_open_sky",
    "sky_tier2_open_sky",
    "sky_tier3_scan",
];

/// Per-tick counts for the lighting hot path. `sky_column_read` is O(height)
/// per `checkBlock`; `get_sky`/`block_state`/`chunk_loaded` are 6x per
/// propagated cell. Logged sorted by count from [`LightPassStats`].
struct LightCounters([AtomicU64; 17]);

impl LightCounters {
    const CHECK_BLOCK: usize = 0;
    const CHECK_SKY: usize = 1;
    const SKY_COLUMN_SCAN: usize = 2;
    const SKY_COLUMN_READ: usize = 3;
    const SKY_INCREASE: usize = 4;
    const SKY_DECREASE: usize = 5;
    const BLOCK_INCREASE: usize = 6;
    const BLOCK_DECREASE: usize = 7;
    const GET_SKY: usize = 8;
    const SET_SKY: usize = 9;
    const GET_BLOCK_LIGHT: usize = 10;
    const SET_BLOCK_LIGHT: usize = 11;
    const BLOCK_STATE: usize = 12;
    const CHUNK_LOADED: usize = 13;
    const SKY_TIER1: usize = 14;
    const SKY_TIER2: usize = 15;
    const SKY_TIER3: usize = 16;

    const fn new() -> Self {
        Self([
            AtomicU64::new(0),
            AtomicU64::new(0),
            AtomicU64::new(0),
            AtomicU64::new(0),
            AtomicU64::new(0),
            AtomicU64::new(0),
            AtomicU64::new(0),
            AtomicU64::new(0),
            AtomicU64::new(0),
            AtomicU64::new(0),
            AtomicU64::new(0),
            AtomicU64::new(0),
            AtomicU64::new(0),
            AtomicU64::new(0),
            AtomicU64::new(0),
            AtomicU64::new(0),
            AtomicU64::new(0),
        ])
    }

    fn bump(&self, index: usize) {
        self.0[index].fetch_add(1, Ordering::Relaxed);
    }

    fn bump_n(&self, index: usize, n: u64) {
        if n > 0 {
            self.0[index].fetch_add(n, Ordering::Relaxed);
        }
    }

    fn snapshot_and_reset(&self) -> [u64; 17] {
        let mut out = [0u64; 17];
        for (i, slot) in self.0.iter().enumerate() {
            out[i] = slot.swap(0, Ordering::Relaxed);
        }
        out
    }
}

/// One `runUpdates` slice. `hot` is sorted most-used first.
#[derive(Clone, Copy)]
pub struct LightPassStats {
    pub elapsed: Duration,
    pub updates: i32,
    pub leftover: bool,
    counts: [u64; 17],
}

impl LightPassStats {
    fn hot_pairs(&self) -> Vec<(&'static str, u64)> {
        let mut items: Vec<(&'static str, u64)> = LIGHT_COUNTER_NAMES
            .iter()
            .zip(self.counts.iter())
            .filter_map(|(name, count)| (*count > 0).then_some((*name, *count)))
            .collect();
        items.sort_unstable_by_key(|a| std::cmp::Reverse(a.1));
        items
    }

    fn hot_list(&self) -> String {
        self.hot_pairs()
            .into_iter()
            .map(|(name, count)| format!("{name}={count}"))
            .collect::<Vec<_>>()
            .join(" ")
    }

    #[must_use]
    pub const fn should_log(&self) -> bool {
        self.leftover
            || self.updates > 0
            || self.elapsed.as_millis() >= 1
            || self.counts[LightCounters::SKY_COLUMN_READ] > 256
    }
}

impl fmt::Display for LightPassStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let hot = self.hot_list();
        if hot.is_empty() {
            write!(
                f,
                "{:?} updates={} leftover={}",
                self.elapsed, self.updates, self.leftover
            )
        } else {
            write!(
                f,
                "{:?} updates={} leftover={} hot: {hot}",
                self.elapsed, self.updates, self.leftover
            )
        }
    }
}

enum VerticalInChunk {
    Below,
    Inside {
        section_index: usize,
        y_in_section: usize,
        local_x: usize,
        local_z: usize,
    },
    Above,
}

const fn vertical_in_chunk(chunk: &ChunkData, pos: &BlockPos) -> VerticalInChunk {
    let (_, relative) = pos.chunk_and_chunk_relative_position();
    let rel_y = relative.y - chunk.section.min_y;
    if rel_y < 0 {
        return VerticalInChunk::Below;
    }
    let section_index = (rel_y as usize) / BlockPalette::SIZE;
    if section_index >= chunk.section.count {
        return VerticalInChunk::Above;
    }
    VerticalInChunk::Inside {
        section_index,
        y_in_section: (rel_y as usize) % BlockPalette::SIZE,
        local_x: relative.x as usize,
        local_z: relative.z as usize,
    }
}

/// Memoized chunk handle for one lighting operation.
///
/// Every `level.read_chunk_sync` is a `DashMap` lookup: hash, shard `RwLock`, table probe,
/// `Arc` deref -> several potential cache misses and an atomic. A single
/// sky propagation step touches 6 neighbours, each with "loaded?", read, opacity and
/// write, so up to 24 such lookups -> and at least two thirds of them land in the
/// same chunk as the origin position. The cursor turns those into a compare plus
/// a pointer deref.
///
/// Deliberately holds the `Arc<ChunkData>` and not the `DashMap` guard: keeping a shard read
/// guard alive across further lookups can deadlock against a waiting writer on the same
/// shard (`parking_lot` lets `read()` block as soon as a writer is queued).
struct ChunkCursor<'a> {
    level: &'a Level,
    counters: &'a LightCounters,
    memo: Option<(Vector2<i32>, Option<Arc<ChunkData>>)>,
}

impl<'a> ChunkCursor<'a> {
    const fn new(level: &'a Level, counters: &'a LightCounters) -> Self {
        Self {
            level,
            counters,
            memo: None,
        }
    }

    fn chunk_at(&mut self, chunk_pos: Vector2<i32>) -> Option<&Arc<ChunkData>> {
        if !matches!(&self.memo, Some((cached, _)) if *cached == chunk_pos) {
            let chunk = self
                .level
                .loaded_chunks
                .get(&chunk_pos)
                .map(|entry| entry.value().clone());
            self.memo = Some((chunk_pos, chunk));
        }
        self.memo.as_ref().and_then(|(_, chunk)| chunk.as_ref())
    }

    fn chunk_for(&mut self, pos: &BlockPos) -> Option<&Arc<ChunkData>> {
        let (chunk_pos, _) = pos.chunk_and_chunk_relative_position();
        self.chunk_at(chunk_pos)
    }

    fn is_loaded(&mut self, pos: &BlockPos) -> bool {
        self.counters.bump(LightCounters::CHUNK_LOADED);
        self.chunk_for(pos).is_some()
    }

    /// Vanilla `getOpacity` is `max(1, getLightDampening())`. No
    /// `useShapeForLightOcclusion` / face voxels: slabs, stairs, trapdoors, leaves
    /// can leak or block unlike vanilla.
    fn opacity(&mut self, pos: &BlockPos) -> u8 {
        self.counters.bump(LightCounters::BLOCK_STATE);
        self.block_state(pos).opacity
    }

    fn block_state(&mut self, pos: &BlockPos) -> &'static pumpkin_data::BlockState {
        let (_, relative) = pos.chunk_and_chunk_relative_position();
        let id = self
            .chunk_for(pos)
            .and_then(|chunk| {
                chunk
                    .section
                    .get_block_absolute_y(relative.x as usize, relative.y, relative.z as usize)
            })
            .unwrap_or(pumpkin_data::Block::VOID_AIR.default_state.id);
        id.to_state()
    }

    fn sky_light(&mut self, pos: &BlockPos) -> u8 {
        self.counters.bump(LightCounters::GET_SKY);
        let Some(chunk) = self.chunk_for(pos) else {
            return 0;
        };
        match vertical_in_chunk(chunk, pos) {
            // Vanilla: sky below the world is 0, above the world is 15.
            VerticalInChunk::Below => 0,
            VerticalInChunk::Above => 15,
            VerticalInChunk::Inside {
                section_index,
                y_in_section,
                local_x,
                local_z,
            } => chunk
                .light_engine
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .sky_light
                .get(section_index)
                .map_or(15, |s| s.get(local_x, y_in_section, local_z)),
        }
    }

    fn block_light(&mut self, pos: &BlockPos) -> Option<u8> {
        self.counters.bump(LightCounters::GET_BLOCK_LIGHT);
        let chunk = self.chunk_for(pos)?;
        let VerticalInChunk::Inside {
            section_index,
            y_in_section,
            local_x,
            local_z,
        } = vertical_in_chunk(chunk, pos)
        else {
            return None;
        };
        chunk
            .light_engine
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .block_light
            .get(section_index)
            .map(|section| section.get(local_x, y_in_section, local_z))
    }

    /// `false` if the write cannot land (chunk not loaded, Y outside the
    /// chunk height). Callers must not re-queue such positions.
    fn set_sky_light(&mut self, pos: &BlockPos, light_level: u8) -> bool {
        self.counters.bump(LightCounters::SET_SKY);
        Self::write_light(self.chunk_for(pos), pos, light_level, false)
    }

    fn set_block_light(&mut self, pos: &BlockPos, light_level: u8) -> bool {
        self.counters.bump(LightCounters::SET_BLOCK_LIGHT);
        Self::write_light(self.chunk_for(pos), pos, light_level, true)
    }

    fn write_light(
        chunk: Option<&Arc<ChunkData>>,
        pos: &BlockPos,
        light_level: u8,
        block_light: bool,
    ) -> bool {
        let Some(chunk) = chunk else {
            return false;
        };
        let VerticalInChunk::Inside {
            section_index,
            y_in_section,
            local_x,
            local_z,
        } = vertical_in_chunk(chunk, pos)
        else {
            return false;
        };
        let mut light_engine = chunk
            .light_engine
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let sections = if block_light {
            &mut light_engine.block_light
        } else {
            &mut light_engine.sky_light
        };
        let Some(section) = sections.get_mut(section_index) else {
            return false;
        };
        section.set(local_x, y_in_section, local_z, light_level);
        drop(light_engine);

        if !chunk.is_dirty() {
            chunk.mark_dirty(true);
        }
        true
    }
}

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
        self.counters.bump(LightCounters::SKY_COLUMN_SCAN);
        // The whole column is in the same chunk by definition: one lookup for `max_y`,
        // after that the cursor carries the chunk through every scan step.
        let Some(max_y) = cursor.chunk_for(pos).map(|chunk| {
            chunk.section.min_y + (chunk.section.count as i32) * BlockPalette::SIZE as i32 - 1
        }) else {
            return false;
        };

        let mut y = pos.0.y;
        let mut reads = 0u64;
        while y < max_y {
            y += 1;
            reads += 1;
            let opacity = cursor.opacity(&BlockPos::new(pos.0.x, y, pos.0.z));
            if opacity > 0 {
                self.counters.bump_n(LightCounters::SKY_COLUMN_READ, reads);
                return false;
            }
        }
        self.counters.bump_n(LightCounters::SKY_COLUMN_READ, reads);

        true
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
            return tier; // Falls schon ohne Grenze unklar, spart das den Nachbar-Lookup.
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
        // Alle drei Schritte arbeiten am selben Chunk; ein Cursor spart deren Lookups.
        let mut cursor = ChunkCursor::new(level, &self.counters);
        self.check_block_light_updates_with(&mut cursor, pos);
        // Must run before the sky pass: the pass reads the cut height this may invalidate.
        Self::refresh_sky_cut_after_change(&mut cursor, &pos);
        self.check_sky_light_updates_with(&mut cursor, pos);
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
            // sit in the same chunk, so the hit rate climbs beyond what a single
            // Einzeloperation hinaus.
            let mut cursor = ChunkCursor::new(level, &self.counters);
            updates += self.perform_block_light_updates(&mut cursor, &mut budget);
            updates += self.perform_sky_light_updates(&mut cursor, &mut budget);
        }
        let stats = LightPassStats {
            elapsed: start.elapsed(),
            updates,
            leftover: !self.queues_empty(),
            counts: self.counters.snapshot_and_reset(),
        };
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
            self.counters.bump(LightCounters::BLOCK_DECREASE);
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
            self.counters.bump(LightCounters::BLOCK_INCREASE);
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

            if let Some(neighbor_light) = cursor.block_light(&neighbor_pos) {
                let opacity = cursor.opacity(&neighbor_pos).max(1);
                let new_light = light_level.saturating_sub(opacity);

                // Only propagate if new light is brighter than current light
                if new_light > neighbor_light
                    && cursor.set_block_light(&neighbor_pos, new_light)
                    && new_light > 1
                {
                    self.queue_block_light_increase(neighbor_pos, new_light);
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

                if let Some(neighbor_light) = cursor.block_light(&neighbor_pos) {
                    if neighbor_light == 0 {
                        continue; // Skip if already 0
                    }

                    let neighbor_state = {
                        self.counters.bump(LightCounters::BLOCK_STATE);
                        cursor.block_state(&neighbor_pos)
                    };
                    let opacity = neighbor_state.opacity.max(1);

                    let expected_from_removed_source = removed_light_level.saturating_sub(opacity);

                    if neighbor_light <= expected_from_removed_source {
                        let neighbor_luminance = neighbor_state.luminance;

                        if neighbor_luminance == 0 {
                            // No self-emission, darken it completely and continue propagation
                            cursor.set_block_light(&neighbor_pos, 0);
                            self.queue_block_light_decrease(neighbor_pos, neighbor_light);
                        } else {
                            // Has self-emission, set to its own light and re-propagate from it
                            cursor.set_block_light(&neighbor_pos, neighbor_luminance);
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
        self.check_block_light_updates_with(&mut cursor, pos);
    }

    fn check_block_light_updates_with(&self, cursor: &mut ChunkCursor, pos: BlockPos) {
        self.counters.bump(LightCounters::CHECK_BLOCK);
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

        let current_light = cursor.block_light(&pos).unwrap_or(0);
        let expected_light = {
            self.counters.bump(LightCounters::BLOCK_STATE);
            cursor.block_state(&pos).luminance
        };

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
            self.check_neighbors_light_updates_with(cursor, pos, expected_light);
        }
    }

    pub fn check_neighbors_light_updates(
        &self,
        level: &Arc<Level>,
        pos: BlockPos,
        current_light: u8,
    ) {
        let mut cursor = ChunkCursor::new(level, &self.counters);
        self.check_neighbors_light_updates_with(&mut cursor, pos, current_light);
    }

    fn check_neighbors_light_updates_with(
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
            self.counters.bump(LightCounters::SKY_DECREASE);
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
            self.counters.bump(LightCounters::SKY_INCREASE);
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
            if !cursor.is_loaded(&neighbor_pos) {
                continue;
            }

            let neighbor_light = cursor.sky_light(&neighbor_pos);
            let opacity = cursor.opacity(&neighbor_pos);

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
            if new_light > neighbor_light
                && cursor.set_sky_light(&neighbor_pos, new_light)
                && new_light > 0
            {
                self.queue_sky_light_increase(neighbor_pos, new_light);
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

            if !cursor.is_loaded(&neighbor_pos) {
                continue;
            }

            let neighbor_light = cursor.sky_light(&neighbor_pos);
            if neighbor_light == 0 {
                continue; // Already dark
            }

            let opacity = cursor.opacity(&neighbor_pos);

            // Calculate what we would have given this neighbor
            let expected = if removed_light == 15 && dir == BlockDirection::Down && opacity == 0 {
                15
            } else {
                removed_light.saturating_sub(1).saturating_sub(opacity)
            };

            if neighbor_light == expected || neighbor_light < removed_light {
                // This neighbor was lit by us, darken it. Skip if the write
                // cannot land (below `min_y` used to stay at sky=15 and loop).
                if cursor.set_sky_light(&neighbor_pos, 0) {
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
        self.check_sky_light_updates_with(&mut cursor, pos);
    }

    fn check_sky_light_updates_with(&self, cursor: &mut ChunkCursor, pos: BlockPos) {
        self.counters.bump(LightCounters::CHECK_SKY);
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

        let current_light = cursor.sky_light(&pos);
        let opacity = cursor.opacity(&pos);

        // Calculate expected sky light
        let expected_light = if opacity == 15 {
            // Fully opaque block = no light
            0
        } else {
            // Check if there's open sky above, cheaply where the cut height can decide it
            let has_sky = match Self::sky_tier(cursor, &pos) {
                SkyLightTier::NoOpenSky => {
                    self.counters.bump(LightCounters::SKY_TIER1);
                    false
                }
                SkyLightTier::OpenSky => {
                    self.counters.bump(LightCounters::SKY_TIER2);
                    true
                }
                SkyLightTier::Unknown => {
                    self.counters.bump(LightCounters::SKY_TIER3);
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
