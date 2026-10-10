use std::{num::NonZeroUsize, time::Duration};

use crate::{SchedulerError, TaskHandle, TaskRequest};

/// Bounds all admitted work, including tasks waiting on external futures
#[derive(Clone, Copy, Debug)]
pub struct SchedulerConfig {
    maximum_tasks: NonZeroUsize,
    turns_per_poll: NonZeroUsize,
    slow_turn_threshold: Duration,
}

impl SchedulerConfig {
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SchedulerState {
    Accepting,
    Stopped,
}

/// Admitted tasks and cumulative terminal outcomes
///
/// Ready counts exclude running tasks, including tasks that wake themselves
/// during polling
/// Terminal counters are updated after cleanup and before result delivery
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
    /// Returns the cumulative successful completions, including detached tasks
    #[must_use]
    pub const fn completed(self) -> u64 {
        self.completed
    }

    /// Returns the cumulative cancelled task count
    #[must_use]
    pub const fn cancelled(self) -> u64 {
        self.cancelled
    }

    /// Returns the cumulative errors, panics, and tasks stopped by driver loss
    #[must_use]
    pub const fn failed(self) -> u64 {
        self.failed
    }

    #[must_use]
    pub const fn ready(self) -> usize {
        self.ready
    }
    #[must_use]
    pub const fn running(self) -> usize {
        self.running
    }
    #[must_use]
    pub const fn pending(self) -> usize {
        self.pending
    }
    #[must_use]
    pub const fn slow_turns(self) -> u64 {
        self.slow_turns
    }
}

/// Object-safe submission interface for tasks returning no value
pub trait SchedulerService: Send + Sync + 'static {
    fn config(&self) -> SchedulerConfig;
    fn state(&self) -> SchedulerState;
    fn snapshot(&self) -> SchedulerSnapshot;
    fn submit(&self, request: TaskRequest) -> Result<TaskHandle, SchedulerError>;
}
