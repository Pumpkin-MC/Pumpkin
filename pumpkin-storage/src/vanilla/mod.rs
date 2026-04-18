use std::path::{Path, PathBuf};

mod level_info;

#[allow(unused_imports)]
pub use level_info::{LEVEL_DAT_BACKUP_FILE_NAME, LEVEL_DAT_FILE_NAME};

/// Filesystem-backed storage laid out the way vanilla Minecraft expects.
///
/// `base_dir` is the world root (the directory containing `level.dat`,
/// `playerdata/`, `region/`, etc.). Domain traits implemented on this struct
/// translate their operations into file I/O under that root.
#[derive(Debug, Clone)]
pub struct VanillaStorage {
    base_dir: PathBuf,
}

impl VanillaStorage {
    pub fn new(base_dir: impl Into<PathBuf>) -> Self {
        Self {
            base_dir: base_dir.into(),
        }
    }

    pub fn base_dir(&self) -> &Path {
        &self.base_dir
    }
}
