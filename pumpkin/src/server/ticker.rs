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
            let _ = server
                .plugin_manager
                .fire(ServerTickStartEvent::new(tick_number))
                .await;

            // Run the tick body in a child task so a panic in world/player tick
            // cannot kill this loop and leave a silently frozen world.
            let server_tick = server.clone();
            let tick_join = tokio::spawn(async move {
                let manager = &server_tick.tick_rate_manager;
                if manager.is_sprinting() {
                    manager.start_sprint_tick_work();
                    server_tick.tick().await;

                    if manager.end_sprint_tick_work() {
                        manager.finish_tick_sprint(&server_tick);
                    }
                } else {
                    server_tick.tick().await;
                }
            });
            if let Err(e) = tick_join.await {
                tracing::error!(
                    "Server tick panicked (#{tick_number}); world tick continues next interval: {e}"
                );
            }

            let tick_duration_nanos = tick_start_time.elapsed().as_nanos() as i64;

            let tick_number = server.tick_count.load(Ordering::Relaxed);
            let _ = server
                .plugin_manager
                .fire(ServerTickEndEvent::new(tick_number, tick_duration_nanos))
                .await;

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
