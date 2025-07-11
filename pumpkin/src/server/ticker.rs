use crate::{SHOULD_STOP, server::Server};
use std::{
    sync::{Arc, atomic::Ordering},
    time::{Duration, Instant},
};
use tokio::time::sleep;

pub struct Ticker {
    last_tick: Instant,
}

impl Default for Ticker {
    fn default() -> Self {
        Self::new()
    }
}

impl Ticker {
    #[must_use]
    pub fn new() -> Self {
        Self {
            last_tick: Instant::now(),
        }
    }

    /// IMPORTANT: Run this in a new thread/tokio task.
    pub async fn run(&mut self, server: &Arc<Server>) {
        while !SHOULD_STOP.load(Ordering::Relaxed) {
            let tick_start_time = Instant::now();
            let manager = &server.tick_rate_manager;

            manager.tick();

            // Now server.tick() handles both player/network ticking (always)
            // and world logic ticking (conditionally based on freeze state)
            if manager.is_sprinting() {
                // A sprint is active, so we tick.
                manager.start_sprint_tick_work();
                server.tick().await;

                // After ticking, end the work and check if the sprint is over.
                if manager.end_sprint_tick_work() {
                    // This was the last sprint tick. Finish the sprint and restore the previous state.
                    manager.finish_tick_sprint(server).await;
                }
            } else {
                // Always call tick - it will internally decide what to tick based on frozen state
                server.tick().await;
            }

            // Record the total time this tick took
            let tick_duration_nanos = tick_start_time.elapsed().as_nanos() as i64;
            server.update_tick_times(tick_duration_nanos).await;

            // Sleep logic remains the same
            let now = Instant::now();
            let elapsed = now.duration_since(self.last_tick);

            let tick_interval = if manager.is_sprinting() {
                Duration::ZERO
            } else {
                Duration::from_nanos(manager.nanoseconds_per_tick() as u64)
            };

            if let Some(sleep_time) = tick_interval.checked_sub(elapsed) {
                if !sleep_time.is_zero() {
                    sleep(sleep_time).await;
                }
            }

            self.last_tick = Instant::now();
        }
        log::debug!("Ticker stopped");
    }
}
