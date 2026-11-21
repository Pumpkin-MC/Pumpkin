use std::{num::NonZeroUsize, time::Duration};

use crate::{SchedulerError, TaskHandle, TaskRequest};

/// Controls admission capacity and how long the driver polls ready work.
///
/// Capacity counts tasks rather than bytes. Suspended tasks still occupy slots;
/// callers must separately bound the memory and external work those tasks retain.
#[derive(Clone, Copy, Debug)]
pub struct SchedulerConfig {
    maximum_tasks: NonZeroUsize,
    turns_per_poll: NonZeroUsize,
    slow_turn_threshold: Duration,
}

impl SchedulerConfig {
    /// Sets the task limit, turn budget, and slow-poll warning threshold.
    ///
    /// `maximum_tasks` includes ready, running, and suspended tasks, whether they
    /// are roots or children. `turns_per_poll` limits how many task polls the
    /// driver performs before yielding to its executor. `slow_turn_threshold`
    /// records and warns about polls taking at least that long; it is not a
    /// timeout and cannot interrupt a running poll.
    #[must_use]
    pub const fn new(
        maximum_tasks: NonZeroUsize,
        turns_per_poll: NonZeroUsize,
        slow_turn_threshold: Duration,
    ) -> Self {
        Self {
            maximum_tasks,
            turns_per_poll,
            slow_turn_threshold,
        }
    }
    #[must_use]
    pub const fn maximum_tasks(self) -> NonZeroUsize {
        self.maximum_tasks
    }
    #[must_use]
    pub const fn turns_per_poll(self) -> NonZeroUsize {
        self.turns_per_poll
    }
    #[must_use]
    pub const fn slow_turn_threshold(self) -> Duration {
        self.slow_turn_threshold
    }
}

/// Whether the scheduler's driver is still accepting submissions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SchedulerState {
    /// The driver is active; submissions remain subject to admission checks.
    Accepting,
    /// The driver has been dropped and new submissions are rejected.
    Stopped,
}

/// A snapshot of admitted work and cumulative task outcomes.
///
/// Ready, running, and pending are separate counts. A running task is excluded
/// from ready even if it wakes itself during its poll. Tasks leave admission
/// before cleanup, and terminal counters are updated after cleanup, before
/// delivery to the result handle. A task being cleaned up can therefore be
/// absent from both the admitted counts and the terminal totals.
#[derive(Clone, Copy, Debug)]
pub struct SchedulerSnapshot {
    pub(crate) ready: usize,
    pub(crate) running: usize,
    pub(crate) pending: usize,
    pub(crate) slow_turns: u64,
    pub(crate) completed: u64,
    pub(crate) cancelled: u64,
    pub(crate) failed: u64,
}

impl SchedulerSnapshot {
    /// Counts successful tasks, including tasks whose result handles were dropped.
    #[must_use]
    pub const fn completed(self) -> u64 {
        self.completed
    }

    /// Counts tasks that completed with a cancellation error naming their own ID.
    ///
    /// A parent that propagates a child's cancellation error counts as failed
    /// unless the parent itself was also cancelled.
    #[must_use]
    pub const fn cancelled(self) -> u64 {
        self.cancelled
    }

    /// Counts task errors, panics, and tasks stopped when the driver was dropped.
    ///
    /// Cancellation errors naming a different task also count as failures.
    #[must_use]
    pub const fn failed(self) -> u64 {
        self.failed
    }

    /// Counts admitted tasks queued for a turn, excluding the running task.
    #[must_use]
    pub const fn ready(self) -> usize {
        self.ready
    }
    /// Counts tasks currently being polled; a Global scheduler has at most one.
    #[must_use]
    pub const fn running(self) -> usize {
        self.running
    }
    /// Counts admitted tasks waiting for a wakeup rather than queued for a turn.
    #[must_use]
    pub const fn pending(self) -> usize {
        self.pending
    }
    /// Counts polls that reached the configured slow-turn threshold.
    #[must_use]
    pub const fn slow_turns(self) -> u64 {
        self.slow_turns
    }
}

/// Submits tasks returning `()` through a trait object.
///
/// Use [`GlobalScheduler::submit`](crate::GlobalScheduler::submit) directly for
/// tasks that return another type.
pub trait SchedulerService: Send + Sync + 'static {
    /// Returns the limits used for task admission and driver polling.
    fn config(&self) -> SchedulerConfig;
    /// Reports whether the driver still accepts work.
    fn state(&self) -> SchedulerState;
    /// Returns current task counts and cumulative completion totals.
    fn snapshot(&self) -> SchedulerSnapshot;
    /// Queues a task without waiting for capacity and returns its result handle.
    ///
    /// # Errors
    ///
    /// Returns the scheduler's admission error if the request cannot be accepted.
    /// Execution and cleanup failures arrive through the returned handle.
    fn submit(&self, request: TaskRequest) -> Result<TaskHandle, SchedulerError>;
}
