//! Hot-path counters. Local `Cell` tally, folded into atomics on drop.

use std::cell::Cell;
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

macro_rules! light_counters {
    ($($variant:ident => $name:literal,)+) => {
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

pub(super) struct LightCounters([AtomicU64; COUNTER_COUNT]);

impl LightCounters {
    pub(super) const fn new() -> Self {
        Self([const { AtomicU64::new(0) }; COUNTER_COUNT])
    }

    pub(super) fn snapshot_and_reset(&self) -> [u64; COUNTER_COUNT] {
        let mut out = [0u64; COUNTER_COUNT];
        for (slot, out) in self.0.iter().zip(out.iter_mut()) {
            *out = slot.swap(0, Ordering::Relaxed);
        }
        out
    }
}

pub(super) struct LocalCounters<'a> {
    local: [Cell<u64>; COUNTER_COUNT],
    shared: &'a LightCounters,
}

impl<'a> LocalCounters<'a> {
    pub(super) const fn new(shared: &'a LightCounters) -> Self {
        Self {
            local: [const { Cell::new(0) }; COUNTER_COUNT],
            shared,
        }
    }

    pub(super) fn bump(&self, counter: Counter) {
        let slot = &self.local[counter as usize];
        slot.set(slot.get() + 1);
    }

    pub(super) fn bump_n(&self, counter: Counter, n: u64) {
        if n > 0 {
            let slot = &self.local[counter as usize];
            slot.set(slot.get() + n);
        }
    }
}

impl Drop for LocalCounters<'_> {
    fn drop(&mut self) {
        for (slot, shared) in self.local.iter().zip(self.shared.0.iter()) {
            let value = slot.get();
            if value != 0 {
                shared.fetch_add(value, Ordering::Relaxed);
            }
        }
    }
}

#[derive(Clone, Copy)]
pub struct LightPassStats {
    pub elapsed: Duration,
    pub updates: i32,
    pub leftover: bool,
    counts: [u64; COUNTER_COUNT],
}

impl LightPassStats {
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

    fn hot_list(&self) -> String {
        let mut items: Vec<(&'static str, u64)> = Counter::NAMES
            .iter()
            .zip(self.counts.iter())
            .filter_map(|(name, count)| (*count > 0).then_some((*name, *count)))
            .collect();
        items.sort_unstable_by_key(|a| std::cmp::Reverse(a.1));
        items
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
