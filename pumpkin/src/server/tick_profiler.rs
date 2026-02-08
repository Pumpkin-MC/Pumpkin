use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Instant;

/// Per-tick phase timing for performance diagnostics.
///
/// Tracks how long each phase of the server tick takes, enabling
/// identification of bottlenecks. When enabled, records timing for
/// the world tick phase and the player/network tick phase separately.
///
/// The profiler is lock-free and uses atomic operations throughout,
/// making it safe to use from the hot tick path without contention.
pub struct TickProfiler {
    enabled: AtomicBool,

    // Current tick timing (written during tick, read by diagnostics)
    phase_start: crossbeam::atomic::AtomicCell<Instant>,

    // Rolling statistics over the last WINDOW_SIZE ticks
    world_tick_nanos: RollingAverage,
    player_tick_nanos: RollingAverage,
    total_tick_nanos: RollingAverage,

    // Slow tick tracking
    slow_tick_threshold_ms: AtomicU64,
    slow_tick_count: AtomicU64,
}

/// A lock-free rolling average over a fixed window.
///
/// Uses atomic operations for concurrent reads during diagnostics
/// while the tick loop writes new samples.
struct RollingAverage {
    samples: [AtomicU64; Self::WINDOW_SIZE],
    index: AtomicU64,
    sum: AtomicU64,
}

impl RollingAverage {
    const WINDOW_SIZE: usize = 100;

    fn new() -> Self {
        Self {
            samples: std::array::from_fn(|_| AtomicU64::new(0)),
            index: AtomicU64::new(0),
            sum: AtomicU64::new(0),
        }
    }

    fn record(&self, value_nanos: u64) {
        let idx = self.index.fetch_add(1, Ordering::Relaxed) as usize % Self::WINDOW_SIZE;
        let old = self.samples[idx].swap(value_nanos, Ordering::Relaxed);
        // Update sum: add new, subtract old
        self.sum.fetch_add(value_nanos, Ordering::Relaxed);
        self.sum.fetch_sub(old, Ordering::Relaxed);
    }

    fn average_nanos(&self) -> u64 {
        let count = self
            .index
            .load(Ordering::Relaxed)
            .min(Self::WINDOW_SIZE as u64);
        if count == 0 {
            return 0;
        }
        self.sum.load(Ordering::Relaxed) / count
    }

    fn last_nanos(&self) -> u64 {
        let idx = self.index.load(Ordering::Relaxed);
        if idx == 0 {
            return 0;
        }
        let last_idx = (idx - 1) as usize % Self::WINDOW_SIZE;
        self.samples[last_idx].load(Ordering::Relaxed)
    }

    fn peak_nanos(&self) -> u64 {
        let count = self
            .index
            .load(Ordering::Relaxed)
            .min(Self::WINDOW_SIZE as u64) as usize;
        let mut peak = 0u64;
        for i in 0..count {
            let val = self.samples[i].load(Ordering::Relaxed);
            if val > peak {
                peak = val;
            }
        }
        peak
    }

    fn sample_count(&self) -> u64 {
        self.index.load(Ordering::Relaxed)
    }
}

/// Snapshot of profiling data for diagnostics display.
#[derive(Debug, Clone)]
pub struct TickProfileSnapshot {
    /// Average world tick duration in nanoseconds (last 100 ticks)
    pub world_avg_nanos: u64,
    /// Average player/network tick duration in nanoseconds (last 100 ticks)
    pub player_avg_nanos: u64,
    /// Average total tick duration in nanoseconds (last 100 ticks)
    pub total_avg_nanos: u64,
    /// Last world tick duration in nanoseconds
    pub world_last_nanos: u64,
    /// Last player/network tick duration in nanoseconds
    pub player_last_nanos: u64,
    /// Last total tick duration in nanoseconds
    pub total_last_nanos: u64,
    /// Peak world tick duration in nanoseconds (last 100 ticks)
    pub world_peak_nanos: u64,
    /// Peak player/network tick duration in nanoseconds (last 100 ticks)
    pub player_peak_nanos: u64,
    /// Peak total tick duration in nanoseconds (last 100 ticks)
    pub total_peak_nanos: u64,
    /// Number of ticks that exceeded the slow tick threshold
    pub slow_tick_count: u64,
    /// Total ticks profiled
    pub total_ticks: u64,
}

impl TickProfileSnapshot {
    /// World tick average in milliseconds.
    #[must_use]
    pub fn world_avg_ms(&self) -> f64 {
        self.world_avg_nanos as f64 / 1_000_000.0
    }

    /// Player/network tick average in milliseconds.
    #[must_use]
    pub fn player_avg_ms(&self) -> f64 {
        self.player_avg_nanos as f64 / 1_000_000.0
    }

    /// Total tick average in milliseconds.
    #[must_use]
    pub fn total_avg_ms(&self) -> f64 {
        self.total_avg_nanos as f64 / 1_000_000.0
    }

    /// Percentage of tick budget (50ms) used on average.
    #[must_use]
    pub fn budget_usage_percent(&self) -> f64 {
        (self.total_avg_nanos as f64 / 50_000_000.0) * 100.0
    }
}

impl Default for TickProfiler {
    fn default() -> Self {
        Self::new()
    }
}

impl TickProfiler {
    /// The default slow tick threshold: 50ms (one full tick at 20 TPS).
    const DEFAULT_SLOW_THRESHOLD_MS: u64 = 50;

    #[must_use]
    pub fn new() -> Self {
        Self {
            enabled: AtomicBool::new(false),
            phase_start: crossbeam::atomic::AtomicCell::new(Instant::now()),
            world_tick_nanos: RollingAverage::new(),
            player_tick_nanos: RollingAverage::new(),
            total_tick_nanos: RollingAverage::new(),
            slow_tick_threshold_ms: AtomicU64::new(Self::DEFAULT_SLOW_THRESHOLD_MS),
            slow_tick_count: AtomicU64::new(0),
        }
    }

    /// Enable or disable profiling.
    pub fn set_enabled(&self, enabled: bool) {
        self.enabled.store(enabled, Ordering::Relaxed);
    }

    /// Whether profiling is currently enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    /// Set the threshold (in milliseconds) above which a tick is considered "slow".
    pub fn set_slow_threshold_ms(&self, ms: u64) {
        self.slow_tick_threshold_ms.store(ms, Ordering::Relaxed);
    }

    /// Mark the beginning of a phase. Returns the start instant for later use.
    pub fn begin_phase(&self) -> Instant {
        let now = Instant::now();
        self.phase_start.store(now);
        now
    }

    /// Record world tick duration.
    pub fn record_world_tick(&self, start: Instant) {
        if !self.is_enabled() {
            return;
        }
        let elapsed = start.elapsed().as_nanos() as u64;
        self.world_tick_nanos.record(elapsed);
    }

    /// Record player/network tick duration.
    pub fn record_player_tick(&self, start: Instant) {
        if !self.is_enabled() {
            return;
        }
        let elapsed = start.elapsed().as_nanos() as u64;
        self.player_tick_nanos.record(elapsed);
    }

    /// Record total tick duration and check for slow ticks.
    pub fn record_total_tick(&self, start: Instant) {
        if !self.is_enabled() {
            return;
        }
        let elapsed_nanos = start.elapsed().as_nanos() as u64;
        self.total_tick_nanos.record(elapsed_nanos);

        let threshold_nanos = self.slow_tick_threshold_ms.load(Ordering::Relaxed) * 1_000_000;
        if elapsed_nanos > threshold_nanos {
            self.slow_tick_count.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Reset slow tick counter.
    pub fn reset_slow_count(&self) {
        self.slow_tick_count.store(0, Ordering::Relaxed);
    }

    /// Get a snapshot of current profiling data.
    pub fn snapshot(&self) -> TickProfileSnapshot {
        TickProfileSnapshot {
            world_avg_nanos: self.world_tick_nanos.average_nanos(),
            player_avg_nanos: self.player_tick_nanos.average_nanos(),
            total_avg_nanos: self.total_tick_nanos.average_nanos(),
            world_last_nanos: self.world_tick_nanos.last_nanos(),
            player_last_nanos: self.player_tick_nanos.last_nanos(),
            total_last_nanos: self.total_tick_nanos.last_nanos(),
            world_peak_nanos: self.world_tick_nanos.peak_nanos(),
            player_peak_nanos: self.player_tick_nanos.peak_nanos(),
            total_peak_nanos: self.total_tick_nanos.peak_nanos(),
            slow_tick_count: self.slow_tick_count.load(Ordering::Relaxed),
            total_ticks: self.total_tick_nanos.sample_count(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn rolling_average_empty() {
        let avg = RollingAverage::new();
        assert_eq!(avg.average_nanos(), 0);
        assert_eq!(avg.last_nanos(), 0);
        assert_eq!(avg.peak_nanos(), 0);
        assert_eq!(avg.sample_count(), 0);
    }

    #[test]
    fn rolling_average_single_sample() {
        let avg = RollingAverage::new();
        avg.record(1_000_000); // 1ms
        assert_eq!(avg.average_nanos(), 1_000_000);
        assert_eq!(avg.last_nanos(), 1_000_000);
        assert_eq!(avg.peak_nanos(), 1_000_000);
        assert_eq!(avg.sample_count(), 1);
    }

    #[test]
    fn rolling_average_multiple_samples() {
        let avg = RollingAverage::new();
        avg.record(2_000_000); // 2ms
        avg.record(4_000_000); // 4ms
        // Average should be 3ms
        assert_eq!(avg.average_nanos(), 3_000_000);
        assert_eq!(avg.last_nanos(), 4_000_000);
        assert_eq!(avg.peak_nanos(), 4_000_000);
        assert_eq!(avg.sample_count(), 2);
    }

    #[test]
    fn rolling_average_wraps_around() {
        let avg = RollingAverage::new();
        // Fill all 100 slots with 1ms each
        for _ in 0..100 {
            avg.record(1_000_000);
        }
        assert_eq!(avg.average_nanos(), 1_000_000);

        // Now add a 101st sample of 2ms, replacing the first 1ms
        avg.record(2_000_000);
        // Sum should now be 99 * 1ms + 1 * 2ms = 101ms, count capped at 100
        assert_eq!(avg.average_nanos(), 1_010_000);
    }

    #[test]
    fn profiler_disabled_by_default() {
        let profiler = TickProfiler::new();
        assert!(!profiler.is_enabled());
    }

    #[test]
    fn profiler_enable_disable() {
        let profiler = TickProfiler::new();
        profiler.set_enabled(true);
        assert!(profiler.is_enabled());
        profiler.set_enabled(false);
        assert!(!profiler.is_enabled());
    }

    #[test]
    fn profiler_records_when_enabled() {
        let profiler = TickProfiler::new();
        profiler.set_enabled(true);

        let start = Instant::now();
        thread::sleep(Duration::from_millis(1));
        profiler.record_world_tick(start);

        let snap = profiler.snapshot();
        assert!(snap.world_avg_nanos > 0);
        assert!(snap.total_ticks == 0); // total_tick not recorded yet
    }

    #[test]
    fn profiler_ignores_when_disabled() {
        let profiler = TickProfiler::new();
        // Profiler is disabled by default

        let start = Instant::now();
        thread::sleep(Duration::from_millis(1));
        profiler.record_world_tick(start);

        let snap = profiler.snapshot();
        assert_eq!(snap.world_avg_nanos, 0);
    }

    #[test]
    fn slow_tick_detection() {
        let profiler = TickProfiler::new();
        profiler.set_enabled(true);
        profiler.set_slow_threshold_ms(0); // Any tick is "slow"

        let start = Instant::now();
        thread::sleep(Duration::from_millis(1));
        profiler.record_total_tick(start);

        let snap = profiler.snapshot();
        assert_eq!(snap.slow_tick_count, 1);
    }

    #[test]
    fn slow_tick_reset() {
        let profiler = TickProfiler::new();
        profiler.set_enabled(true);
        profiler.set_slow_threshold_ms(0);

        let start = Instant::now();
        profiler.record_total_tick(start);

        profiler.reset_slow_count();
        let snap = profiler.snapshot();
        assert_eq!(snap.slow_tick_count, 0);
    }

    #[test]
    fn snapshot_budget_usage() {
        let snap = TickProfileSnapshot {
            world_avg_nanos: 0,
            player_avg_nanos: 0,
            total_avg_nanos: 25_000_000, // 25ms
            world_last_nanos: 0,
            player_last_nanos: 0,
            total_last_nanos: 0,
            world_peak_nanos: 0,
            player_peak_nanos: 0,
            total_peak_nanos: 0,
            slow_tick_count: 0,
            total_ticks: 100,
        };
        assert!((snap.budget_usage_percent() - 50.0).abs() < 0.01);
        assert!((snap.total_avg_ms() - 25.0).abs() < 0.01);
    }
}
