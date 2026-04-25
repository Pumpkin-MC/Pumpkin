use crate::{SHOULD_STOP, server::Server};
#[cfg(windows)]
use spin_sleep::{SpinSleeper, SpinStrategy};
use std::sync::atomic::Ordering;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::runtime::Handle;
use tracing::debug;

pub struct Ticker;

#[cfg(windows)]
fn sleep_until_tick(deadline: Instant) {
    SpinSleeper::new(100_000)
        .with_spin_strategy(SpinStrategy::YieldThread)
        .sleep_until(deadline);
}

#[cfg(not(windows))]
fn sleep_until_tick(deadline: Instant) {
    if let Some(sleep_time) = deadline.checked_duration_since(Instant::now())
        && !sleep_time.is_zero()
    {
        std::thread::sleep(sleep_time);
    }
}

impl Ticker {
    pub fn run(server: &Arc<Server>, handle: &Handle) {
        let mut next_tick = Instant::now();
        loop {
            if SHOULD_STOP.load(Ordering::Relaxed) {
                break;
            }

            let tick_start_time = Instant::now();
            let manager = &server.tick_rate_manager;

            manager.tick();

            handle.block_on(async {
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
            });

            let tick_interval = if manager.is_sprinting() {
                Duration::ZERO
            } else {
                Duration::from_nanos(manager.nanoseconds_per_tick() as u64)
            };

            if tick_interval.is_zero() {
                next_tick = Instant::now();
                continue;
            }

            next_tick += tick_interval;

            if next_tick <= Instant::now() {
                next_tick = Instant::now();
            } else {
                sleep_until_tick(next_tick);
            }
        }
        debug!("Ticker stopped");
    }
}
