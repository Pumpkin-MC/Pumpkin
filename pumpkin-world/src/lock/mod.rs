use std::path::Path;

use thiserror::Error;

pub mod anvil;

// Gets unlocked when dropped
pub trait LevelLocker<T>: Send + Sync {
    fn lock(folder: &Path) -> Result<T, LockError>;
}

#[derive(Error, Debug)]
pub enum LockError {
    #[error("Oh no, Level is already locked by {0}")]
    AlreadyLocked(String),
    #[error("Failed to write into lock file")]
    FailedWrite,
}
