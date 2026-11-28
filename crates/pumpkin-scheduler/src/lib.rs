//! Runs owned async tasks through a bounded Global scheduler.
//!
//! The application supplies a [`TaskExecutor`] to poll the scheduler's driver;
//! this crate does not create a runtime or worker threads. Only
//! [`ExecutionDomain::Global`] is supported. Other domain variants describe
//! future routing targets.
//!
//! Ready tasks receive one poll at a time in FIFO order. When a task suspends,
//! another ready task can run, including a child whose result the parent awaits.
//! This orders individual polls, not complete operations. Release locks needed
//! by other tasks before awaiting, and revalidate shared state after resuming.
//! Blocking I/O and long CPU jobs must run outside the driver because it cannot
//! interrupt a task while that task is being polled.
//!
//! # Task lifetime
//!
//! Submission either reserves a slot immediately or returns an error. Ready,
//! running, and suspended tasks all count against the same capacity, including
//! child tasks. [`TaskContext`] preserves task ancestry across suspension, but
//! does not grant access to game state or Wasm stores; callers manage that access.
//!
//! Dropping a [`TaskHandle`] stops observing its result and leaves the work
//! running. Use a [`CancellationHandle`] to cancel a task and its active
//! descendants. Cancellation takes effect between polls and does not undo side
//! effects or stop work started on another executor. Await the task's handle to
//! observe its cleanup and result. Descendants have their own completion handles.
//!
//! The application's executor owns the driver. Dropping the driver stops new
//! submissions and settles outstanding tasks; graceful draining is not provided.
//!
//! # Nested work
//!
//! Submit children with the parent's context to preserve the call chain and
//! cancellation relationship. A suspended parent still occupies a slot, so
//! child submission can fail when the scheduler is full.
//!
//! ```
//! use pumpkin_scheduler::{ExecutionDomain, GlobalScheduler, SchedulerError, TaskRequest};
//!
//! async fn nested_work(scheduler: GlobalScheduler) -> Result<u32, SchedulerError> {
//!     let children = scheduler.clone();
//!     scheduler.submit(TaskRequest::new(ExecutionDomain::Global, move |parent| async move {
//!         children.submit(TaskRequest::child(&parent, |_child| async { Ok(42) }))?.await
//!     }))?.await
//! }
//! ```

mod backend;
mod cancellation;
mod completion;
mod domain;
mod error;
mod global;
mod scheduler;
mod task;

pub use backend::{ExecutorFuture, TaskExecutor};
pub use cancellation::CancellationHandle;
pub use domain::{
    EntityDomainId, ExecutionDomain, ExternalDomainId, RegionDomainId, WorldDomainId,
};
pub use error::SchedulerError;
pub use global::GlobalScheduler;
pub use scheduler::{SchedulerConfig, SchedulerService, SchedulerSnapshot, SchedulerState};
pub use task::{SchedulerTaskId, TaskContext, TaskFuture, TaskHandle, TaskRequest, TaskWork};
