use super::server_test_manager::drain_game_test_queue;

use crate::{
    STOP_INTERRUPT,
    plugin::server::{
        server_tick_end::ServerTickEndEvent, server_tick_start::ServerTickStartEvent,
    },
    server::Server,
};
use pumpkin_gametest::GameTestRunner;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};
use tracing::{debug, warn};

const NANOSECONDS_PER_MILLISECOND: i64 = 1_000_000;
const NANOSECONDS_PER_SECOND: i64 = 1_000_000_000;
/// Vanilla `OVERLOADED_THRESHOLD_NANOS` (`20 * 1s / 20` = 1s).
const OVERLOADED_THRESHOLD_NANOS: i64 = 20 * NANOSECONDS_PER_SECOND / 20;
/// Vanilla `OVERLOADED_TICKS_THRESHOLD`.
const OVERLOADED_TICKS_THRESHOLD: i64 = 20;
/// Vanilla `OVERLOADED_WARNING_INTERVAL_NANOS`.
const OVERLOADED_WARNING_INTERVAL_NANOS: i64 = 10 * NANOSECONDS_PER_SECOND;
/// Vanilla `OVERLOADED_TICKS_WARNING_INTERVAL`.
const OVERLOADED_TICKS_WARNING_INTERVAL: i64 = 100;

pub struct Ticker;

impl Ticker {
    /// Runs the main server tick loop on a dedicated thread.
    pub fn run(server: &Arc<Server>) {
        let _guard = server.runtime.enter();
        let mut next_tick = Instant::now();
        // Vanilla `lastOverloadWarningNanos` starts at 0, so the first overload
        // always passes the warning-interval check.
        let mut last_overload_warning: Option<Instant> = None;
        let mut game_test_runner = GameTestRunner::new();

        let park_thread = std::thread::current();
        let stop = STOP_INTERRUPT.clone();
        server.runtime.spawn(async move {
            stop.cancelled().await;
            park_thread.unpark();
        });

        'ticker: loop {
            let now = Instant::now();
            let manager = &server.tick_rate_manager;
            let sprinting = manager.is_sprinting();
            let this_tick_nanos = if sprinting {
                0
            } else {
                manager.nanoseconds_per_tick()
            };

            // Vanilla `MinecraftServer.runServer`: skip ticks when more than
            // `OVERLOADED_THRESHOLD + 20 * nanosecondsPerTick` behind (~2s at 20 TPS).
            let (scheduled, warning, skipped) =
                apply_overload_skip(now, next_tick, last_overload_warning, this_tick_nanos);
            next_tick = scheduled;
            last_overload_warning = warning;
            if let Some((behind_ms, ticks)) = skipped {
                warn!(
                    "Can't keep up! Is the server overloaded? Running {behind_ms}ms or {ticks} ticks behind"
                );
            }

            // Deadline for the next wait. Work below may finish late; then the
            // following iteration catch-up-runs with no park.
            next_tick += nanos_to_duration(this_tick_nanos);

            let tick_start_time = Instant::now();

            manager.tick();

            let tick_number = server.tick_count.load(Ordering::Relaxed);
            if server.plugin_manager.has_handlers::<ServerTickStartEvent>() {
                server.runtime.block_on(
                    server
                        .plugin_manager
                        .fire(server, &mut ServerTickStartEvent::new(tick_number)),
                );
            }

            let should_tick_game_tests = manager.runs_normally() || sprinting;

            if sprinting {
                manager.start_sprint_tick_work();
                server.tick();

                if manager.end_sprint_tick_work() {
                    manager.finish_tick_sprint(server);
                }
            } else {
                server.tick();
            }

            if should_tick_game_tests {
                server.runtime.block_on(async {
                    drain_game_test_queue(server, &mut game_test_runner).await;
                    game_test_runner.tick().await;
                });
            }

            let tick_duration_nanos = tick_start_time.elapsed().as_nanos() as i64;

            let tick_number = server.tick_count.load(Ordering::Relaxed);
            if server.plugin_manager.has_handlers::<ServerTickEndEvent>() {
                server.runtime.block_on(server.plugin_manager.fire(
                    server,
                    &mut ServerTickEndEvent::new(tick_number, tick_duration_nanos),
                ));
            }

            server.update_tick_times(tick_duration_nanos);

            if STOP_INTERRUPT.is_cancelled() {
                break 'ticker;
            }

            wait_until_next_tick(next_tick);

            if STOP_INTERRUPT.is_cancelled() {
                break 'ticker;
            }
        }

        debug!("Ticker stopped");
    }
}

/// Vanilla `MinecraftServer` overload skip. Returns the (possibly jumped)
/// `nextTickTimeNanos`, updated warning timestamp, and an optional log payload.
///
/// `this_tick_nanos <= 0` is a sprint tick: reset the deadline to `now`.
fn apply_overload_skip(
    now: Instant,
    next_tick: Instant,
    last_overload_warning: Option<Instant>,
    this_tick_nanos: i64,
) -> (Instant, Option<Instant>, Option<(i64, i64)>) {
    if this_tick_nanos <= 0 {
        return (now, Some(now), None);
    }

    let behind_nanos = signed_nanos(now, next_tick);
    let since_warning =
        last_overload_warning.map_or(i64::MAX, |warned| signed_nanos(next_tick, warned));

    if behind_nanos > OVERLOADED_THRESHOLD_NANOS + OVERLOADED_TICKS_THRESHOLD * this_tick_nanos
        && since_warning
            >= OVERLOADED_WARNING_INTERVAL_NANOS
                + OVERLOADED_TICKS_WARNING_INTERVAL * this_tick_nanos
    {
        let ticks = behind_nanos / this_tick_nanos;
        let jumped = next_tick + nanos_to_duration(ticks * this_tick_nanos);
        let behind_ms = behind_nanos / NANOSECONDS_PER_MILLISECOND;
        return (jumped, Some(jumped), Some((behind_ms, ticks)));
    }

    (next_tick, last_overload_warning, None)
}

/// Vanilla `waitUntilNextTick` / `LockSupport.parkNanos`.
/// TODO drain chunk/main-thread work here (vanilla `pollTask`) until the deadline.
fn wait_until_next_tick(next_tick: Instant) {
    loop {
        if STOP_INTERRUPT.is_cancelled() {
            return;
        }

        let now = Instant::now();
        if now >= next_tick {
            return;
        }

        std::thread::park_timeout(next_tick - now);
    }
}

fn signed_nanos(later: Instant, earlier: Instant) -> i64 {
    if later >= earlier {
        later.duration_since(earlier).as_nanos() as i64
    } else {
        -(earlier.duration_since(later).as_nanos() as i64)
    }
}

fn nanos_to_duration(nanos: i64) -> Duration {
    Duration::from_nanos(nanos.max(0) as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TICK_20_TPS: i64 = NANOSECONDS_PER_SECOND / 20;

    fn at(origin: Instant, nanos: i64) -> Instant {
        origin + nanos_to_duration(nanos)
    }

    #[test]
    fn on_time_tick_does_not_skip() {
        let origin = Instant::now();
        let (next, warning, skipped) = apply_overload_skip(origin, origin, None, TICK_20_TPS);
        assert!(skipped.is_none());
        assert_eq!(signed_nanos(next, origin), 0);
        assert!(warning.is_none());
    }

    #[test]
    fn short_lag_catch_up_does_not_skip() {
        let origin = Instant::now();
        let now = at(origin, 60 * NANOSECONDS_PER_MILLISECOND);
        let (next, _, skipped) = apply_overload_skip(now, origin, None, TICK_20_TPS);
        assert!(skipped.is_none());
        assert_eq!(signed_nanos(next, origin), 0);
    }

    #[test]
    fn overload_warning_is_rate_limited() {
        let origin = Instant::now();
        let now = at(origin, 3 * NANOSECONDS_PER_SECOND);
        let just_warned = Some(origin);
        let (_, _, skipped) = apply_overload_skip(now, origin, just_warned, TICK_20_TPS);
        assert!(skipped.is_none());
    }

    #[test]
    fn sprint_resets_deadline_to_now() {
        let origin = Instant::now();
        let now = at(origin, NANOSECONDS_PER_SECOND);
        let (next, warning, skipped) = apply_overload_skip(now, origin, None, 0);
        assert!(skipped.is_none());
        assert_eq!(next, now);
        assert_eq!(warning, Some(now));
    }
}
