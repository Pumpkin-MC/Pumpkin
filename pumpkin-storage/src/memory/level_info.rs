//! In-memory [`LevelInfoStorage`] holding a single `Option<LevelData>`.

use crate::BoxFuture;
use crate::error::StorageError;
use crate::level_info::{LevelData, LevelInfoStorage};
use crate::memory::MemoryStorage;

impl LevelInfoStorage for MemoryStorage {
    fn load(&self) -> BoxFuture<'_, Result<LevelData, StorageError>> {
        Box::pin(async move {
            let guard = self.level_info.read().await;
            guard.clone().ok_or_else(|| StorageError::NotFound {
                message: "level info not stored".to_string(),
            })
        })
    }

    fn save<'a>(&'a self, data: &'a LevelData) -> BoxFuture<'a, Result<(), StorageError>> {
        Box::pin(async move {
            *self.level_info.write().await = Some(data.clone());
            Ok(())
        })
    }
}
