//! Per-pass instrumentation for the lighting hot path.
//!
//! Split out of `runtime` so the engine file holds propagation logic only. The counters
//! are plain indices into one array rather than named fields: the hot path bumps them by
//! index, and the array is snapshotted and reset in one sweep per pass.

use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

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
pub(super) struct LightCounters([AtomicU64; 17]);

impl LightCounters {
    pub(super) const CHECK_BLOCK: usize = 0;
    pub(super) const CHECK_SKY: usize = 1;
    pub(super) const SKY_COLUMN_SCAN: usize = 2;
    pub(super) const SKY_COLUMN_READ: usize = 3;
    pub(super) const SKY_INCREASE: usize = 4;
    pub(super) const SKY_DECREASE: usize = 5;
    pub(super) const BLOCK_INCREASE: usize = 6;
    pub(super) const BLOCK_DECREASE: usize = 7;
    pub(super) const GET_SKY: usize = 8;
    pub(super) const SET_SKY: usize = 9;
    pub(super) const GET_BLOCK_LIGHT: usize = 10;
    pub(super) const SET_BLOCK_LIGHT: usize = 11;
    pub(super) const BLOCK_STATE: usize = 12;
    pub(super) const CHUNK_LOADED: usize = 13;
    pub(super) const SKY_TIER1: usize = 14;
    pub(super) const SKY_TIER2: usize = 15;
    pub(super) const SKY_TIER3: usize = 16;

    pub(super) const fn new() -> Self {
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

    pub(super) fn bump(&self, index: usize) {
        self.0[index].fetch_add(1, Ordering::Relaxed);
    }

    pub(super) fn bump_n(&self, index: usize, n: u64) {
        if n > 0 {
            self.0[index].fetch_add(n, Ordering::Relaxed);
        }
    }

    pub(super) fn snapshot_and_reset(&self) -> [u64; 17] {
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
    /// Built by [`super::DynamicLightEngine::drain_queued`]; `counts` stays private so the
    /// counter layout is not part of the public surface.
    pub(super) const fn new(
        elapsed: Duration,
        updates: i32,
        leftover: bool,
        counts: [u64; 17],
    ) -> Self {
        Self {
            elapsed,
            updates,
            leftover,
            counts,
        }
    }

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
