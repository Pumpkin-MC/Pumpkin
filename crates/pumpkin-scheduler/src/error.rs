use thiserror::Error;

use crate::{ExecutionDomain, SchedulerTaskId};

/// An admission failure or the terminal error returned by an accepted task.
#[derive(Debug, Error)]
pub enum SchedulerError {
    /// The named task was cancelled, or a child was submitted to a cancelled parent.
    #[error("scheduler task {task:?} was cancelled")]
    Cancelled { task: SchedulerTaskId },
    #[error("scheduler capacity for {domain:?} is full")]
    QueueFull { domain: ExecutionDomain },
    #[error("scheduler does not support domain {domain:?}")]
    UnsupportedDomain { domain: ExecutionDomain },
    #[error("parent task belongs to a different scheduler")]
    ForeignContext,
    #[error("parent task {task:?} is no longer active or valid")]
    InactiveParent { task: SchedulerTaskId },
    #[error("scheduler task identifiers exhausted")]
    TaskIdsExhausted,
    #[error("scheduler task {task:?} panicked: {message}")]
    TaskPanicked {
        task: SchedulerTaskId,
        message: String,
    },
    #[error("scheduler driver stopped")]
    Stopped,
    #[error("scheduler executor failed: {0}")]
    Backend(String),
}
