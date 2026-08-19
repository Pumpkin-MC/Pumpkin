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

            let tick_interval = if manager.is_sprinting() {
                Duration::ZERO
            } else {
                Duration::from_nanos(manager.nanoseconds_per_tick() as u64)
            };

            next_tick += tick_interval;

            // Explicitly yield to tokio to allow pending network packets / I/O tasks to be processed
            tokio::task::yield_now().await;

            tokio::select! {
                () = sleep_until(next_tick) => {},
                () = STOP_INTERRUPT.cancelled() => {
                    break 'ticker;
                }
            }

            // Death Spiral Prevention / Catch-up Clamping
            // If the server fell behind the scheduled tick, clamp next_tick to now
            // so we don't run a burst of back-to-back ticks with zero sleep.
            let now = Instant::now();
            if now > next_tick {
                next_tick = now;
            }
        }

        debug!("Ticker stopped");
    }
}
