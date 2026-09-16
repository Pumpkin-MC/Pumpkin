use std::{
    fmt,
    sync::{
        Arc, Weak,
        atomic::{AtomicBool, Ordering},
    },
};

use crate::{SchedulerTaskId, global::Shared};

/// Requests cancellation of a task and its active descendants
///
/// Accepted work continues when result or cancellation handles are dropped
/// Clones share the request flag and hold a weak reference to the scheduler
///
/// Cancellation propagates through the selected task's active descendants
/// Parents and siblings remain unaffected
/// Cancellation requires the selected task to remain active
///
/// The driver drops cancelled futures between polls
/// A running poll continues until it returns and existing side effects remain
/// External work requires cancellation through its own executor
/// Task state must remain valid when a future is dropped at a suspension point
#[derive(Clone)]
pub struct CancellationHandle {
    shared: Weak<Shared>,
    task: SchedulerTaskId,
    requested: Arc<AtomicBool>,
}

impl CancellationHandle {
    pub(crate) fn new(shared: &Arc<Shared>, task: SchedulerTaskId) -> Self {
        Self {
            shared: Arc::downgrade(shared),
            task,
            requested: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Requests cancellation and wakes the driver without waiting for cleanup
    ///
    /// Returns `true` for the first accepted request while the task is admitted
    /// and the driver is accepting work
    /// New child submissions are then rejected and active descendants are cancelled
    ///
    /// Cancellation and completion are ordered by the queue lock
    /// Cancellation accepted before completion produces
    /// [`SchedulerError::Cancelled`](crate::SchedulerError::Cancelled), even if
    /// the current poll returns a value
    /// A cleanup panic produces a failure
    /// Returns `false` if completion has already been committed
    ///
    /// Awaiting the result handle observes cleanup of this task
    /// Descendant cleanup can be observed through each descendant's result handle
    #[must_use]
    pub fn cancel(&self) -> bool {
        self.shared
            .upgrade()
            .is_some_and(|shared| shared.cancel(self.task))
    }

    /// Checks whether this task or an active ancestor requested cancellation
    ///
    /// The flag remains `true` after completion
    /// Cleanup completion is observed through the result handle
    #[must_use]
    pub fn is_requested(&self) -> bool {
        self.requested.load(Ordering::Acquire)
    }

    pub(crate) fn request(&self) {
        self.requested.store(true, Ordering::Release);
    }
}

impl fmt::Debug for CancellationHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CancellationHandle")
            .field("task", &self.task)
            .field("requested", &self.is_requested())
            .finish_non_exhaustive()
    }
}
