use std::{future::Future, pin::Pin};

use crate::SchedulerError;

pub type ExecutorFuture = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

/// Runs the Global domain scheduler driver on an executor supplied by the application
///
/// An accepted future must be scheduled for polling and woken normally. The
/// executor must not synchronously run it to completion inside `spawn`. Dropping
/// the driver closes admission and resolves outstanding handles with an error
pub trait TaskExecutor: Send + Sync + 'static {
    fn spawn(&self, future: ExecutorFuture) -> Result<(), SchedulerError>;
}
