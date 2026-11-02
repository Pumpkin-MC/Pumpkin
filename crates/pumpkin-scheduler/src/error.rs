use thiserror::Error;

use crate::{ExecutionDomain, SchedulerTaskId};

#[derive(Debug, Error)]
pub enum SchedulerError {
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
