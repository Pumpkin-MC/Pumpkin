use tokio::sync::RwLock;

use crate::level_info::LevelData;

mod level_info;

/// Format-agnostic, in-memory storage.
///
/// Unlike [`VanillaStorage`](crate::VanillaStorage), this backend holds domain
/// values directly (no serialization, no on-disk layout). Intended for tests,
/// ephemeral servers, and embedded contexts where persistence is not needed.
#[derive(Debug, Default)]
pub struct MemoryStorage {
    pub(crate) level_info: RwLock<Option<LevelData>>,
}

impl MemoryStorage {
    pub fn new() -> Self {
        Self::default()
    }
}
