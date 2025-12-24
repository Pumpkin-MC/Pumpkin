use super::{LevelLocker, LockError};

use std::{
    fs::{File, TryLockError},
    io::Write,
    path::Path,
};

pub struct AnvilLevelLocker {
    _locked_file: File,
}

const SESSION_LOCK_FILE_NAME: &str = "session.lock";

const SNOWMAN: &[u8] = "☃".as_bytes();

impl LevelLocker<Self> for AnvilLevelLocker {
    fn lock(folder: &Path) -> Result<Self, LockError> {
        std::fs::create_dir_all(folder).map_err(|_| LockError::FailedWrite)?;
        let file_path = folder.join(SESSION_LOCK_FILE_NAME);
        let mut file = File::options()
            .create(true)
            .truncate(false)
            .write(true)
            .open(file_path)
            .unwrap();
        // im not joking, mojang writes a snowman into the lock file
        file.write_all(SNOWMAN)
            .map_err(|_| LockError::FailedWrite)?;
        file.try_lock().map_err(|e| match e {
            TryLockError::WouldBlock => {
                LockError::AlreadyLocked(SESSION_LOCK_FILE_NAME.to_string())
            }
            TryLockError::Error(io_err) => LockError::Error(io_err),
        })?;
        Ok(Self { _locked_file: file })
    }
}
