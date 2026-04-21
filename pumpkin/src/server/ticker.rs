use crate::{STOP_INTERRUPT, server::Server};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::{Instant, sleep_until};
use tracing::debug;

pub struct Ticker;

impl Ticker {
    /// IMPORTANT: Run this in a new thread/tokio task.
    pub async fn run(server: &Arc<Server>) {
        let mut next_tick = Instant::now();

        'ticker: loop {
            let tick_start_time = std::time::Instant::now();
            let manager = &server.tick_rate_manager;

            manager.tick();

            if manager.is_sprinting() {
                manager.start_sprint_tick_work();
                server.tick().await;

                if manager.end_sprint_tick_work() {
                    manager.finish_tick_sprint(server).await;
                }
            } else {
                server.tick().await;
            }

            let tick_duration_nanos = tick_start_time.elapsed().as_nanos() as i64;
            server.update_tick_times(tick_duration_nanos).await;

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
