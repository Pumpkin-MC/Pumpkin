use crate::chunk::ChunkData;
use crate::chunk::io::Dirtiable;
use crate::chunk::palette::BlockPalette;
use crate::level::Level;
use crate::lighting::sky_light_height::{
    QUADRANT_DIVERGENCE_THRESHOLD, SkyLightHeight, SkyLightHeightMigration, SkyLightTier,
};
use crossbeam::queue::SegQueue;
use pumpkin_config::lighting::LightingEngineConfig;
use pumpkin_data::BlockDirection;
use pumpkin_util::math::position::BlockPos;
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
    fn has_open_sky_above(&self, level: &Arc<Level>, pos: &BlockPos) -> bool {
        self.counters.bump(LightCounters::SKY_COLUMN_SCAN);
        let (chunk_pos, _) = pos.chunk_and_chunk_relative_position();
        let Some(max_y) = level.read_chunk_sync(&chunk_pos, |chunk| {
            chunk.section.min_y + (chunk.section.count as i32) * BlockPalette::SIZE as i32 - 1
        }) else {
            return false;
        };

        let mut y = pos.0.y;
        let mut reads = 0u64;
        while y < max_y {
            y += 1;
            reads += 1;
            let opacity = self.block_opacity(level, &BlockPos::new(pos.0.x, y, pos.0.z));
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
    fn sky_tier(level: &Arc<Level>, pos: &BlockPos) -> SkyLightTier {
        let (chunk_pos, relative) = pos.chunk_and_chunk_relative_position();
        level
            .read_chunk_sync(&chunk_pos, |chunk| {
                let height = SkyLightHeightMigration::get(chunk);
                height.tier(
                    pos.0.y,
                    relative.x,
                    relative.z,
                    chunk.section.min_y,
                    SkyLightHeight::chunk_height(chunk),
                )
            })
            .unwrap_or(SkyLightTier::Unknown)
    }

    /// Keeps the cached cut height honest after a block change.
    ///
    /// A non-diverged quadrant promises every column ceiling sits in
    /// `[cut, cut + QUADRANT_DIVERGENCE_THRESHOLD]`. Only a change at or above the cut can
    /// break that: digging below the cut leaves the occluders above it untouched, and
    /// placing below the cut cannot raise a ceiling that is already at or above it. So we
    /// re-derive this one column and flag the quadrant if it left the band.
    ///
    /// Differs from the plan, which invalidates on changes *below* the cut: that describes
    /// an already-stale cut, which cannot arise while this runs on every block change.
    fn refresh_sky_cut_after_change(level: &Arc<Level>, pos: &BlockPos) {
        let (chunk_pos, relative) = pos.chunk_and_chunk_relative_position();
        level.read_chunk_sync(&chunk_pos, |chunk| {
            let cached = chunk.sky_light_height_cache.load(Ordering::Relaxed);
            if cached == 0 {
                return; // Never computed: the first compute will see this change anyway.
            }
            let height = SkyLightHeight::from_raw(cached);
            if !height.quadrant_uses_limit(relative.x, relative.z) {
                return; // Already diverged, nothing left to invalidate.
            }
            let cut = height.decode(chunk.section.min_y, SkyLightHeight::chunk_height(chunk));
            if pos.0.y < cut {
                return;
            }

            let ceiling = SkyLightHeight::column_ceiling_at(chunk, relative.x, relative.z);
            if ceiling < cut || ceiling > cut + QUADRANT_DIVERGENCE_THRESHOLD {
                SkyLightHeightMigration::mark_quadrant_diverged(chunk, relative.x, relative.z);
            }
        });
    }

    fn queues_empty(&self) -> bool {
        self.block_decrease.is_empty()
            && self.block_increase.is_empty()
            && self.sky_decrease.is_empty()
            && self.sky_increase.is_empty()
    }

    /// Vanilla `getOpacity` is `max(1, getLightDampening())`. No
    /// `useShapeForLightOcclusion` / face voxels: slabs, stairs, trapdoors, leaves
    /// can leak or block unlike vanilla.
    fn block_opacity(&self, level: &Arc<Level>, pos: &BlockPos) -> u8 {
        self.counters.bump(LightCounters::BLOCK_STATE);
        level.get_block_state(pos).to_state().opacity
    }

    fn chunk_is_loaded(&self, level: &Arc<Level>, pos: &BlockPos) -> bool {
        self.counters.bump(LightCounters::CHUNK_LOADED);
        let (chunk, _) = pos.chunk_and_chunk_relative_position();
        level.is_chunk_loaded(&chunk)
    }

    /// Vanilla `Level.setBlock` -> `LightEngine.checkBlock`: enqueue only.
    /// Flood is [`Self::drain_queued`]. Sync on the tick thread (not the light thread).
    pub fn update_lighting_at(&self, level: &Arc<Level>, pos: BlockPos) {
        let _guard = self
            .propagate_lock
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        self.check_block_light_updates(level, pos);
        // Must run before the sky pass: the pass reads the cut height this may invalidate.
        Self::refresh_sky_cut_after_change(level, &pos);
        self.check_sky_light_updates(level, pos);
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
            updates += self.perform_block_light_updates(level, &mut budget);
            updates += self.perform_sky_light_updates(level, &mut budget);
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

    fn perform_block_light_updates(&self, level: &Arc<Level>, budget: &mut i32) -> i32 {
        let mut updates = 0;

        loop {
            if *budget <= 0 {
                break;
            }
            let decrease_updates = self.perform_block_light_decrease_updates(level, budget);
            let increase_updates = self.perform_block_light_increase_updates(level, budget);

            updates += decrease_updates + increase_updates;

            if decrease_updates == 0 && increase_updates == 0 {
                break;
            }
        }

        updates
    }

    fn perform_block_light_decrease_updates(&self, level: &Arc<Level>, budget: &mut i32) -> i32 {
        let mut updates = 0;

        while *budget > 0 {
            let Some((pos, expected_light)) = self.block_decrease.pop() else {
                break;
            };
            *budget -= 1;
            self.counters.bump(LightCounters::BLOCK_DECREASE);
            self.propagate_block_light_decrease(level, &pos, expected_light);
            updates += 1;
        }

        updates
    }

    fn perform_block_light_increase_updates(&self, level: &Arc<Level>, budget: &mut i32) -> i32 {
        let mut updates = 0;

        while *budget > 0 {
            let Some((pos, expected_light)) = self.block_increase.pop() else {
                break;
            };
            *budget -= 1;
            self.counters.bump(LightCounters::BLOCK_INCREASE);
            self.propagate_block_light_increase(level, &pos, expected_light);
            updates += 1;
        }

        updates
    }

    fn propagate_block_light_increase(&self, level: &Arc<Level>, pos: &BlockPos, light_level: u8) {
        for dir in BlockDirection::all() {
            let neighbor_pos = pos.offset(dir.to_offset());

            if let Some(neighbor_light) = self.get_block_light_level(level, &neighbor_pos) {
                let opacity = self.block_opacity(level, &neighbor_pos).max(1);
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

                    let neighbor_state = {
                        self.counters.bump(LightCounters::BLOCK_STATE);
                        level.get_block_state(&neighbor_pos).to_state()
                    };
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
        self.counters.bump(LightCounters::CHECK_BLOCK);
        match level.lighting_config {
            // Pumpkin config, not vanilla: whole world fullbright / pitch black.
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
        let expected_light = {
            self.counters.bump(LightCounters::BLOCK_STATE);
            level.get_block_state(&pos).to_state().luminance
        };

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

    fn perform_sky_light_updates(&self, level: &Arc<Level>, budget: &mut i32) -> i32 {
        let mut updates = 0;
        loop {
            if *budget <= 0 {
                break;
            }
            let decrease_updates = self.perform_sky_light_decrease_updates(level, budget);
            let increase_updates = self.perform_sky_light_increase_updates(level, budget);

            updates += decrease_updates + increase_updates;

            if decrease_updates == 0 && increase_updates == 0 {
                break;
            }
        }
        updates
    }

    fn perform_sky_light_decrease_updates(&self, level: &Arc<Level>, budget: &mut i32) -> i32 {
        let mut updates = 0;
        while *budget > 0 {
            let Some((pos, expected_light)) = self.sky_decrease.pop() else {
                break;
            };
            *budget -= 1;
            self.counters.bump(LightCounters::SKY_DECREASE);
            self.propagate_sky_light_decrease(level, &pos, expected_light);
            updates += 1;
        }
        updates
    }

    fn perform_sky_light_increase_updates(&self, level: &Arc<Level>, budget: &mut i32) -> i32 {
        let mut updates = 0;
        while *budget > 0 {
            let Some((pos, expected_light)) = self.sky_increase.pop() else {
                break;
            };
            *budget -= 1;
            self.counters.bump(LightCounters::SKY_INCREASE);
            self.propagate_sky_light_increase(level, &pos, expected_light);
            updates += 1;
        }
        updates
    }

    fn propagate_sky_light_increase(&self, level: &Arc<Level>, pos: &BlockPos, light_level: u8) {
        for dir in BlockDirection::all() {
            let neighbor_pos = pos.offset(dir.to_offset());

            // Vanilla missing chunk is `Blocks.BEDROCK` (opaque). Skip here so a
            // dropped write cannot re-queue forever; the seam can stay bright/dark
            // until the neighbour loads.
            if !self.chunk_is_loaded(level, &neighbor_pos) {
                continue;
            }

            let neighbor_light = self.get_sky_light_level(level, &neighbor_pos);
            let opacity = self.block_opacity(level, &neighbor_pos);

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
                && self
                    .set_sky_light_level(level, &neighbor_pos, new_light)
                    .is_ok()
                && new_light > 0
            {
                self.queue_sky_light_increase(neighbor_pos, new_light);
            }
        }
    }

    fn propagate_sky_light_decrease(&self, level: &Arc<Level>, pos: &BlockPos, removed_light: u8) {
        for dir in BlockDirection::all() {
            let neighbor_pos = pos.offset(dir.to_offset());

            if !self.chunk_is_loaded(level, &neighbor_pos) {
                continue;
            }

            let neighbor_light = self.get_sky_light_level(level, &neighbor_pos);
            if neighbor_light == 0 {
                continue; // Already dark
            }

            let opacity = self.block_opacity(level, &neighbor_pos);

            // Calculate what we would have given this neighbor
            let expected = if removed_light == 15 && dir == BlockDirection::Down && opacity == 0 {
                15
            } else {
                removed_light.saturating_sub(1).saturating_sub(opacity)
            };

            if neighbor_light == expected || neighbor_light < removed_light {
                // This neighbor was lit by us, darken it. Skip if the write
                // cannot land (below `min_y` used to stay at sky=15 and loop).
                if self.set_sky_light_level(level, &neighbor_pos, 0).is_ok() {
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
        self.counters.bump(LightCounters::CHECK_SKY);
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
        let opacity = self.block_opacity(level, &pos);

        // Calculate expected sky light
        let expected_light = if opacity == 15 {
            // Fully opaque block = no light
            0
        } else {
            // Check if there's open sky above, cheaply where the cut height can decide it
            let has_sky = match Self::sky_tier(level, &pos) {
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
                    self.has_open_sky_above(level, &pos)
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
        self.counters.bump(LightCounters::GET_BLOCK_LIGHT);
        let (chunk_pos, _) = position.chunk_and_chunk_relative_position();

        level.read_chunk_sync(&chunk_pos, |chunk| {
            let VerticalInChunk::Inside {
                section_index,
                y_in_section,
                local_x,
                local_z,
            } = vertical_in_chunk(chunk, position)
            else {
                return None;
            };
            let light_engine = chunk.light_engine.lock().ok()?;
            light_engine
                .block_light
                .get(section_index)?
                .get(local_x, y_in_section, local_z)
                .into()
        })?
    }

    pub fn get_sky_light_level_sync(&self, level: &Level, position: &BlockPos) -> u8 {
        self.counters.bump(LightCounters::GET_SKY);
        let (chunk_coordinate, _) = position.chunk_and_chunk_relative_position();
        level
            .read_chunk_sync(&chunk_coordinate, |chunk| {
                match vertical_in_chunk(chunk, position) {
                    // Vanilla: sky below the world is 0, above the world is 15.
                    // `as usize` on `y < min_y` used to wrap and look like "above".
                    VerticalInChunk::Below => 0,
                    VerticalInChunk::Above => 15,
                    VerticalInChunk::Inside {
                        section_index,
                        y_in_section,
                        local_x,
                        local_z,
                    } => {
                        let light_engine = chunk
                            .light_engine
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner);
                        light_engine
                            .sky_light
                            .get(section_index)
                            .map_or(15, |s| s.get(local_x, y_in_section, local_z))
                    }
                }
            })
            .unwrap_or(0)
    }

    pub fn get_block_light_level(&self, level: &Arc<Level>, position: &BlockPos) -> Option<u8> {
        self.counters.bump(LightCounters::GET_BLOCK_LIGHT);
        let (chunk_pos, _) = position.chunk_and_chunk_relative_position();

        level
            .read_chunk_sync(&chunk_pos, |chunk| {
                let VerticalInChunk::Inside {
                    section_index,
                    y_in_section,
                    local_x,
                    local_z,
                } = vertical_in_chunk(chunk, position)
                else {
                    return None;
                };
                chunk
                    .light_engine
                    .lock()
                    .ok()?
                    .block_light
                    .get(section_index)
                    .map(|section| section.get(local_x, y_in_section, local_z))
            })
            .flatten()
    }

    pub fn set_block_light_level(
        &self,
        level: &Arc<Level>,
        position: &BlockPos,
        light_level: u8,
    ) -> Result<(), String> {
        self.counters.bump(LightCounters::SET_BLOCK_LIGHT);
        let (chunk_coordinate, _) = position.chunk_and_chunk_relative_position();
        level
            .read_chunk_sync(&chunk_coordinate, |chunk| {
                let VerticalInChunk::Inside {
                    section_index,
                    y_in_section,
                    local_x,
                    local_z,
                } = vertical_in_chunk(chunk, position)
                else {
                    return Err("Y outside chunk height".to_string());
                };
                {
                    let mut light_engine = chunk
                        .light_engine
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    let Some(section) = light_engine.block_light.get_mut(section_index) else {
                        return Err("Invalid section index".to_string());
                    };
                    section.set(local_x, y_in_section, local_z, light_level);
                };
                if !chunk.is_dirty() {
                    chunk.mark_dirty(true);
                }
                Ok(())
            })
            .unwrap_or_else(|| Err("chunk not loaded".to_string()))
    }

    pub fn get_sky_light_level(&self, level: &Arc<Level>, position: &BlockPos) -> u8 {
        self.counters.bump(LightCounters::GET_SKY);
        let (chunk_coordinate, _) = position.chunk_and_chunk_relative_position();
        level
            .read_chunk_sync(&chunk_coordinate, |chunk| {
                match vertical_in_chunk(chunk, position) {
                    VerticalInChunk::Below => 0,
                    VerticalInChunk::Above => 15,
                    VerticalInChunk::Inside {
                        section_index,
                        y_in_section,
                        local_x,
                        local_z,
                    } => {
                        let light_engine = chunk
                            .light_engine
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner);
                        light_engine
                            .sky_light
                            .get(section_index)
                            .map_or(15, |s| s.get(local_x, y_in_section, local_z))
                    }
                }
            })
            .unwrap_or(0)
    }

    pub fn set_sky_light_level(
        &self,
        level: &Arc<Level>,
        position: &BlockPos,
        light_level: u8,
    ) -> Result<(), String> {
        self.counters.bump(LightCounters::SET_SKY);
        let (chunk_coordinate, _) = position.chunk_and_chunk_relative_position();
        level
            .read_chunk_sync(&chunk_coordinate, |chunk| {
                let VerticalInChunk::Inside {
                    section_index,
                    y_in_section,
                    local_x,
                    local_z,
                } = vertical_in_chunk(chunk, position)
                else {
                    return Err("Y outside chunk height".to_string());
                };
                {
                    let mut light_engine = chunk
                        .light_engine
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    let Some(section) = light_engine.sky_light.get_mut(section_index) else {
                        return Err("Invalid section index".to_string());
                    };
                    section.set(local_x, y_in_section, local_z, light_level);
                };
                if !chunk.is_dirty() {
                    chunk.mark_dirty(true);
                }
                Ok(())
            })
            .unwrap_or_else(|| Err("chunk not loaded".to_string()))
    }
}
