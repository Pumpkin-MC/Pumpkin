use std::{
    fmt,
    sync::{
        Arc, Weak,
        atomic::{AtomicBool, Ordering},
    },
};

use crate::{SchedulerTaskId, global::Shared};

/// Allows cancellation of one task and its active descendants.
///
/// Obtain a handle from the task's [`TaskHandle`](crate::TaskHandle) or
/// [`TaskContext`](crate::TaskContext). Clones share the same cancellation flag,
/// but do not keep the scheduler alive. Dropping a handle does not cancel work.
///
/// Cancellation leaves parents, siblings, and unrelated task chains running.
/// The driver drops cancelled futures between polls; it cannot interrupt a
/// running poll, undo committed effects, or stop work on another executor.
/// Task code must therefore leave shared state valid when its future is dropped
/// at a suspension point.
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

    /// Requests cancellation without waiting for the task to stop.
    ///
    /// Returns `true` when this request marks an active task for cancellation.
    /// Its active descendants are marked too, child submissions are rejected,
    /// and the driver is woken to process cleanup. Returns `false` if cancellation
    /// was already requested, the task has finished, or the scheduler has stopped.
    ///
    /// An accepted request takes precedence over the current poll's result,
    /// even if that poll returns successfully. Dropping the future can still
    /// fail with [`SchedulerError::TaskPanicked`](crate::SchedulerError::TaskPanicked).
    ///
    /// Await the task's result handle to observe its cleanup. This does not wait
    /// for descendant cleanup; each descendant has its own result handle.
    #[must_use]
    pub fn cancel(&self) -> bool {
        self.shared
            .upgrade()
            .is_some_and(|shared| shared.cancel(self.task))
    }

    /// Reports whether this task has been marked for cancellation.
    ///
    /// A request on an active ancestor also sets this flag. Once set, it stays
    /// `true` after completion; it does not indicate that cleanup has finished.
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
