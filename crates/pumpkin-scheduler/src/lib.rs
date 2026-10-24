//! Stackless task scheduling for Pumpkin execution domains
//!
//! The application supplies the runtime and executor that poll the scheduler driver.
//! Currently only the [`ExecutionDomain::Global`] is currently supported.
//!
//! Task polls are serialized and suspended tasks allow other work to run.
//! Callers must revalidate state after suspension because call chains can interleave.
//! CPU-heavy work must run on a separate CPU executor.
//!
//! [`TaskContext`] preserves parent and root chain identities across suspension.
//! State locking, Wasm store access, plugin routing, and tick execution remain
//! the responsibility of the caller.
//!
//! Dropping a [`TaskHandle`] leaves accepted work running.
//! Explicit cancellation through [`CancellationHandle`] affects the task and
//! its active descendants.
//!
//! # Examples
//!
//! A parent can await a child submitted with its context.
//! Suspending the parent allows the same driver to poll the child.
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
