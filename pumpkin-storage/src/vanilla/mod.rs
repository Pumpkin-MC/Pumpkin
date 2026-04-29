use std::path::{Path, PathBuf};

/// Filesystem-backed storage laid out the way vanilla Minecraft expects.
///
/// `world_dir` is the world root (the directory containing `level.dat`,
/// `playerdata/`, `region/`, etc.). Domain traits implemented on this struct
/// translate their operations into file I/O under that root. The current PR
/// only wires the `world_info` domain; subsequent PRs add the rest and may
/// introduce additional roots (e.g. a server-wide `data/` for ops/whitelist).
#[derive(Debug, Clone)]
pub struct VanillaStorage {
    world_dir: PathBuf,
}

impl VanillaStorage {
    pub fn new(world_dir: impl Into<PathBuf>) -> Self {
        Self {
            world_dir: world_dir.into(),
        }
    }

    #[must_use]
    pub fn world_dir(&self) -> &Path {
        &self.world_dir
    }
}
