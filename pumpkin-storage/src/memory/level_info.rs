//! In-memory [`LevelInfoStorage`] holding a single `Option<LevelData>`.

use async_trait::async_trait;

use crate::error::StorageError;
use crate::level_info::{LevelData, LevelInfoStorage};
use crate::memory::MemoryStorage;

#[async_trait]
impl LevelInfoStorage for MemoryStorage {
    async fn load(&self) -> Result<LevelData, StorageError> {
        let guard = self.level_info.read().await;
        guard.clone().ok_or_else(|| StorageError::NotFound {
            message: "level info not stored".to_string(),
        })
    }

    async fn save(&self, data: &LevelData) -> Result<(), StorageError> {
        *self.level_info.write().await = Some(data.clone());
        Ok(())
    }
}
