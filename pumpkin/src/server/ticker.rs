use crate::{
    STOP_INTERRUPT,
    plugin::server::{
        server_tick_end::ServerTickEndEvent, server_tick_start::ServerTickStartEvent,
    },
    server::Server,
};
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;
use tokio::time::{Instant, sleep_until};
use tracing::debug;

pub struct Ticker;

impl Ticker {
    /// IMPORTANT: Run this in a new thread/tokio task.
    pub async fn run(server: &Arc<Server>) {
        let mut next_tick = Instant::now();
        let mut last_metrics = std::time::Instant::now();

        'ticker: loop {
            let tick_start_time = std::time::Instant::now();
            let manager = &server.tick_rate_manager;

            manager.tick();

            let tick_number = server.tick_count.load(Ordering::Relaxed);
            server
                .plugin_manager
                .fire(server, &mut ServerTickStartEvent::new(tick_number))
                .await;

            if manager.is_sprinting() {
                manager.start_sprint_tick_work();
                server.tick().await;

                if manager.end_sprint_tick_work() {
                    manager.finish_tick_sprint(server);
                }
            } else {
                server.tick().await;
            }

            let tick_duration_nanos = tick_start_time.elapsed().as_nanos() as i64;

            let tick_number = server.tick_count.load(Ordering::Relaxed);
            server
                .plugin_manager
                .fire(
                    server,
                    &mut ServerTickEndEvent::new(tick_number, tick_duration_nanos),
                )
                .await;

            server.update_tick_times(tick_duration_nanos).await;

            if last_metrics.elapsed() >= Duration::from_secs(1) {
                let tick_count = server.tick_count.load(Ordering::Relaxed);
                let sample_size = (tick_count as usize).min(100);
                if sample_size > 0 {
                    let mut samples = server.get_tick_times_nanos_copy().await;
                    let samples = &mut samples[..sample_size];
                    samples.sort_unstable();
                    let percentile = |fraction: f64| {
                        let index = ((sample_size - 1) as f64 * fraction).round() as usize;
                        samples[index]
                    };
                    let target_nanos = manager.nanoseconds_per_tick();
                    let ticks_over_budget = samples
                        .iter()
                        .filter(|duration| **duration > target_nanos)
                        .count();
                    debug!(
                        target: "pumpkin::tick_metrics",
                        sample_size,
                        p50_ms = percentile(0.50) as f64 / 1_000_000.0,
                        p95_ms = percentile(0.95) as f64 / 1_000_000.0,
                        p99_ms = percentile(0.99) as f64 / 1_000_000.0,
                        max_ms = samples[sample_size - 1] as f64 / 1_000_000.0,
                        ticks_over_budget,
                        target_ms = target_nanos as f64 / 1_000_000.0,
                        "server tick interval"
                    );
                }
                last_metrics = std::time::Instant::now();
            }

            let tick_interval = if manager.is_sprinting() {
                Duration::ZERO
            } else {
                Duration::from_nanos(manager.nanoseconds_per_tick() as u64)
            };

            next_tick += tick_interval;

            tokio::select! {
                () = sleep_until(next_tick) => {},
                () = STOP_INTERRUPT.cancelled() => {
                    break 'ticker;
                }
            }

            // Death Spiral Prevention
            let now = Instant::now();
            if now.saturating_duration_since(next_tick) > Duration::from_secs(5) {
                next_tick = now;
            }
        }

        debug!("Ticker stopped");
    }
}
