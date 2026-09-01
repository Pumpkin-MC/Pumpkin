use crate::{
    STOP_INTERRUPT,
    plugin::server::{
        server_tick_end::ServerTickEndEvent, server_tick_start::ServerTickStartEvent,
    },
    server::Server,
};
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};
use tracing::debug;

pub struct Ticker;

impl Ticker {
    /// Runs the main server tick loop on a dedicated thread.
    pub fn run(server: &Arc<Server>) {
        let _guard = server.runtime.enter();
        let mut next_tick = Instant::now();

        'ticker: loop {
            let tick_start_time = Instant::now();
            let manager = &server.tick_rate_manager;

            manager.tick();

            let tick_number = server.tick_count.load(Ordering::Relaxed);
            if server.plugin_manager.has_handlers::<ServerTickStartEvent>() {
                server.runtime.block_on(
                    server
                        .plugin_manager
                        .fire(server, &mut ServerTickStartEvent::new(tick_number)),
                );
            }

            if manager.is_sprinting() {
                manager.start_sprint_tick_work();
                server.tick();

                if manager.end_sprint_tick_work() {
                    manager.finish_tick_sprint(server);
                }
            } else {
                server.tick();
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

            let tick_interval = if manager.is_sprinting() {
                Duration::ZERO
            } else {
                Duration::from_nanos(manager.nanoseconds_per_tick() as u64)
            };

            next_tick += tick_interval;

            if STOP_INTERRUPT.is_cancelled() {
                break 'ticker;
            }

            let now = Instant::now();
            if next_tick > now {
                let sleep_duration = next_tick - now;
                let cancelled = STOP_INTERRUPT.clone();
                server.runtime.block_on(async {
                    tokio::select! {
                        () = tokio::time::sleep(sleep_duration) => {},
                        () = cancelled.cancelled() => {},
                    }
                });

                if STOP_INTERRUPT.is_cancelled() {
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
