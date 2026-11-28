use std::{future::Future, pin::Pin};

use crate::SchedulerError;

/// The scheduler driver future passed to the application's executor.
pub type ExecutorFuture = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

/// Connects the scheduler driver to the application's async executor.
///
/// An accepted driver must be polled and scheduled again when its waker fires.
/// Retain it for as long as the scheduler should accept work: dropping it stops
/// admission and settles outstanding tasks.
pub trait TaskExecutor: Send + Sync + 'static {
    /// Takes ownership of a driver without waiting for it to finish.
    ///
    /// # Errors
    ///
    /// Return an error if the executor cannot accept the driver.
    fn spawn(&self, future: ExecutorFuture) -> Result<(), SchedulerError>;
}
