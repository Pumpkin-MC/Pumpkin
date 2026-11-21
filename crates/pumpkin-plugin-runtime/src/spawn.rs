use std::{future::Future, pin::Pin};

use thiserror::Error;

pub type SpawnFuture = Pin<Box<dyn Future<Output = ()> + Send + 'static>>;

#[derive(Clone, Debug, Error)]
#[error("{message}")]
pub struct SpawnError {
    message: String,
}

impl SpawnError {
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

/// Spawns runtime work without selecting or constructing an async runtime.
pub trait RuntimeSpawner: Send + Sync + 'static {
    /// Transfers ownership of a future that must eventually be polled or
    /// dropped when spawning succeeds.
    fn spawn(&self, task: SpawnFuture) -> Result<(), SpawnError>;

    fn spawn_blocking(&self, task: Box<dyn FnOnce() + Send + 'static>) -> Result<(), SpawnError>;
}
