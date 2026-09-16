use std::{future::Future, pin::Pin};

use crate::SchedulerError;

pub type ExecutorFuture = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

/// Runs the Global domain scheduler driver on an application-supplied executor
///
/// An accepted future must be scheduled for polling and woken normally
/// The executor must return from `spawn` while the driver remains active
/// Dropping the driver closes admission and resolves outstanding handles with an error
pub trait TaskExecutor: Send + Sync + 'static {
    fn spawn(&self, future: ExecutorFuture) -> Result<(), SchedulerError>;
}
