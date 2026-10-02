use std::sync::Mutex;
use std::time::{Duration, Instant};

#[derive(Clone, Copy)]
struct DebugProfileSession {
    started_at: Instant,
    started_tick: u32,
}

/// Measurements collected by a completed `/debug start` profiling session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DebugProfileResult {
    pub duration: Duration,
    pub ticks: u32,
}

impl DebugProfileResult {
    #[must_use]
    pub fn ticks_per_second(self) -> f64 {
        let seconds = self.duration.as_secs_f64();
        if seconds == 0.0 {
            return 0.0;
        }

        f64::from(self.ticks) / seconds
    }

    #[must_use]
    pub fn command_result(self) -> i32 {
        let floored_tps = self
            .ticks_per_second()
            .floor()
            .clamp(0.0, f64::from(i32::MAX));

        #[expect(clippy::cast_possible_truncation)]
        {
            floored_tps as i32
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartDebugProfileError {
    AlreadyRunning,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StopDebugProfileError {
    NotRunning,
}

/// Owns the single server-wide tick profiling session used by `/debug`.
#[derive(Default)]
pub struct DebugProfiler {
    active_session: Mutex<Option<DebugProfileSession>>,
}

impl DebugProfiler {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn start(&self, current_tick: i32) -> Result<(), StartDebugProfileError> {
        self.start_at(current_tick as u32, Instant::now())
    }

    pub fn stop(&self, current_tick: i32) -> Result<DebugProfileResult, StopDebugProfileError> {
        self.stop_at(current_tick as u32, Instant::now())
    }

    fn start_at(&self, current_tick: u32, now: Instant) -> Result<(), StartDebugProfileError> {
        let mut active_session = self
            .active_session
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        if active_session.is_some() {
            return Err(StartDebugProfileError::AlreadyRunning);
        }

        *active_session = Some(DebugProfileSession {
            started_at: now,
            started_tick: current_tick,
        });
        Ok(())
    }

    fn stop_at(
        &self,
        current_tick: u32,
        now: Instant,
    ) -> Result<DebugProfileResult, StopDebugProfileError> {
        let session = self
            .active_session
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take()
            .ok_or(StopDebugProfileError::NotRunning)?;

        Ok(DebugProfileResult {
            duration: now.saturating_duration_since(session.started_at),
            ticks: current_tick.wrapping_sub(session.started_tick),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{DebugProfiler, StartDebugProfileError, StopDebugProfileError};
    use std::time::{Duration, Instant};

    #[test]
    fn profile_lifecycle_enforces_state_and_reports_measurements() {
        let profiler = DebugProfiler::new();
        let start = Instant::now();

        assert_eq!(
            profiler.stop_at(42, start),
            Err(StopDebugProfileError::NotRunning)
        );
        assert_eq!(profiler.start_at(42, start), Ok(()));
        assert_eq!(
            profiler.start_at(100, start + Duration::from_secs(1)),
            Err(StartDebugProfileError::AlreadyRunning)
        );

        let result = profiler
            .stop_at(62, start + Duration::from_secs(2))
            .expect("the running profile should stop");

        assert_eq!(result.duration, Duration::from_secs(2));
        assert_eq!(result.ticks, 20);
        assert_eq!(result.ticks_per_second(), 10.0);
        assert_eq!(result.command_result(), 10);
        assert_eq!(
            profiler.stop_at(62, start + Duration::from_secs(2)),
            Err(StopDebugProfileError::NotRunning)
        );
    }
}
