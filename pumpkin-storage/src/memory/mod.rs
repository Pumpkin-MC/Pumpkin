use tokio::sync::RwLock;

use crate::world_info::LevelData;

mod world_info;

/// Format-agnostic, in-memory storage.
///
/// Unlike [`VanillaStorage`](crate::VanillaStorage), this backend holds domain
/// values directly (no serialization, no on-disk layout). Intended for tests,
/// ephemeral servers, and embedded contexts where persistence is not needed.
#[derive(Debug, Default)]
pub struct MemoryStorage {
    pub(crate) world_info: RwLock<Option<LevelData>>,
}

impl MemoryStorage {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}
