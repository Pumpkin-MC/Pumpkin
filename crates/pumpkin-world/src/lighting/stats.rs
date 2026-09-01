//! Per-pass instrumentation for the lighting hot path.
//!
//! Split out of `runtime` so the engine file holds propagation logic only. The counters
//! live in one flat array: the hot path bumps them by index, and the array is snapshotted
//! and reset in a single sweep per pass.

use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

/// Declares the counters once and derives everything else from that one list.
///
/// The variants number themselves from zero upwards in declaration order
macro_rules! light_counters {
    ($($variant:ident => $name:literal,)+) => {
        /// One counter per measured operation on the lighting hot path.
        #[derive(Clone, Copy)]
        #[repr(usize)]
        pub(super) enum Counter {
            $($variant,)+
        }

        impl Counter {
            const NAMES: &'static [&'static str] = &[$($name,)+];
        }
    };
}

light_counters! {
    CheckBlock => "check_block",
    CheckSky => "check_sky",
    SkyColumnScan => "sky_column_scan",
    SkyColumnRead => "sky_column_read",
    SkyIncrease => "sky_increase",
    SkyDecrease => "sky_decrease",
    BlockIncrease => "block_increase",
    BlockDecrease => "block_decrease",
    GetSky => "get_sky",
    SetSky => "set_sky",
    GetBlockLight => "get_block_light",
    SetBlockLight => "set_block_light",
    BlockState => "block_state",
    ChunkLoaded => "chunk_loaded",
    SkyTier1 => "sky_tier1_no_open_sky",
    SkyTier2 => "sky_tier2_open_sky",
    SkyTier3 => "sky_tier3_scan",
}

const COUNTER_COUNT: usize = Counter::NAMES.len();

/// Per-tick counts for the lighting hot path. `sky_column_read` is O(height)
/// per `checkBlock`; `get_sky`/`block_state`/`chunk_loaded` are 6x per
/// propagated cell. Logged sorted by count from [`LightPassStats`].
pub(super) struct LightCounters([AtomicU64; COUNTER_COUNT]);

impl LightCounters {
    pub(super) const fn new() -> Self {
        Self([const { AtomicU64::new(0) }; COUNTER_COUNT])
    }

    pub(super) fn bump(&self, counter: Counter) {
        self.0[counter as usize].fetch_add(1, Ordering::Relaxed);
    }

    pub(super) fn bump_n(&self, counter: Counter, n: u64) {
        if n > 0 {
            self.0[counter as usize].fetch_add(n, Ordering::Relaxed);
        }
    }

    pub(super) fn snapshot_and_reset(&self) -> [u64; COUNTER_COUNT] {
        let mut out = [0u64; COUNTER_COUNT];
        for (slot, out) in self.0.iter().zip(out.iter_mut()) {
            *out = slot.swap(0, Ordering::Relaxed);
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
    counts: [u64; COUNTER_COUNT],
}

impl LightPassStats {
    /// Built by [`super::DynamicLightEngine::drain_queued`]; `counts` stays private so the
    /// counter layout is not part of the public surface.
    pub(super) const fn new(
        elapsed: Duration,
        updates: i32,
        leftover: bool,
        counts: [u64; COUNTER_COUNT],
    ) -> Self {
        Self {
            elapsed,
            updates,
            leftover,
            counts,
        }
    }

    /// Reads one counter out of the snapshot. -> only for tests
    #[cfg(test)]
    pub(super) const fn count(&self, counter: Counter) -> u64 {
        self.counts[counter as usize]
    }

    fn hot_pairs(&self) -> Vec<(&'static str, u64)> {
        let mut items: Vec<(&'static str, u64)> = Counter::NAMES
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
            || self.counts[Counter::SkyColumnRead as usize] > 256
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
