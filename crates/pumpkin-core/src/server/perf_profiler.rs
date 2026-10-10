use std::sync::Mutex;
use std::time::{Duration, Instant};

/// How long a `/perf` profiling run lasts unless it is stopped early.
pub const PERF_PROFILE_DURATION: Duration = Duration::from_secs(10);

struct PerfSession {
    started_at: Instant,
    started_tick: i32,
    generation: u64,
}

/// Measurements collected by a completed `/perf` profiling run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PerfProfileResult {
    pub duration: Duration,
    pub ticks: u32,
}

impl PerfProfileResult {
    #[must_use]
    pub fn ticks_per_second(self) -> f64 {
        let seconds = self.duration.as_secs_f64();
        if seconds == 0.0 {
            return 0.0;
        }

        f64::from(self.ticks) / seconds
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartPerfError {
    AlreadyRunning,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StopPerfError {
    NotRunning,
}

#[derive(Default)]
struct PerfState {
    session: Option<PerfSession>,
    next_generation: u64,
}

/// Owns the single server-wide metrics recording session used by `/perf`.
///
/// Every run gets a generation token so the scheduled auto-stop only ends the
/// run it was started for and never a newer one.
#[derive(Default)]
pub struct PerfProfiler {
    state: Mutex<PerfState>,
}

impl PerfProfiler {
    /// Starts a profiling run, returning its generation token.
    pub fn start(&self, current_tick: i32) -> Result<u64, StartPerfError> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        if state.session.is_some() {
            return Err(StartPerfError::AlreadyRunning);
        }

        let generation = state.next_generation;
        state.next_generation = state.next_generation.wrapping_add(1);
        state.session = Some(PerfSession {
            started_at: Instant::now(),
            started_tick: current_tick,
            generation,
        });

        Ok(generation)
    }

    /// Stops the active profiling run, whichever one it is.
    pub fn stop(&self, current_tick: i32) -> Result<PerfProfileResult, StopPerfError> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        let session = state.session.take().ok_or(StopPerfError::NotRunning)?;
        Ok(Self::finish(&session, current_tick))
    }

    /// Stops the profiling run identified by `generation` if it is still the
    /// active one. Used by the auto-stop task so it never ends a newer run.
    #[must_use]
    pub fn stop_if_generation(
        &self,
        generation: u64,
        current_tick: i32,
    ) -> Option<PerfProfileResult> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        if state
            .session
            .as_ref()
            .is_some_and(|session| session.generation == generation)
        {
            let session = state.session.take()?;
            return Some(Self::finish(&session, current_tick));
        }

        None
    }

    fn finish(session: &PerfSession, current_tick: i32) -> PerfProfileResult {
        PerfProfileResult {
            duration: session.started_at.elapsed(),
            ticks: current_tick
                .wrapping_sub(session.started_tick)
                .unsigned_abs(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{PerfProfiler, StartPerfError, StopPerfError};

    #[test]
    fn lifecycle_enforces_single_session() {
        let profiler = PerfProfiler::default();
        assert_eq!(profiler.stop(0), Err(StopPerfError::NotRunning));

        assert_eq!(profiler.start(100), Ok(0));
        assert_eq!(profiler.start(100), Err(StartPerfError::AlreadyRunning));

        assert!(profiler.stop(140).is_ok_and(|result| result.ticks == 40));
        assert_eq!(profiler.stop(140), Err(StopPerfError::NotRunning));

        // The auto-stop of a finished run must not end a newer run.
        assert_eq!(profiler.start(200), Ok(1));
        assert!(profiler.stop_if_generation(0, 220).is_none());
        assert!(
            profiler
                .stop_if_generation(1, 220)
                .is_some_and(|result| result.ticks == 20)
        );
    }
}
