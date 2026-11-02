use std::{num::NonZeroUsize, time::Duration};

use crate::{SchedulerError, TaskHandle, TaskRequest};

/// Bounds all admitted work, including tasks waiting on external futures.
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

/// Counts of admitted tasks. A running task is excluded from ready even if
/// it has already scheduled its next turn by waking itself
#[derive(Clone, Copy, Debug)]
pub struct SchedulerSnapshot {
    pub(crate) ready: usize,
    pub(crate) running: usize,
    pub(crate) pending: usize,
    pub(crate) slow_turns: u64,
}

impl SchedulerSnapshot {
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

/// Object-safe submission surface for tasks returning no value
pub trait SchedulerService: Send + Sync + 'static {
    fn config(&self) -> SchedulerConfig;
    fn state(&self) -> SchedulerState;
    fn snapshot(&self) -> SchedulerSnapshot;
    fn submit(&self, request: TaskRequest) -> Result<TaskHandle, SchedulerError>;
}
