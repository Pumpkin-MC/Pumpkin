//! In-memory [`WorldInfoStorage`] holding a single `Option<LevelData>`.

use crate::BoxFuture;
use crate::error::StorageError;
use crate::memory::MemoryStorage;
use crate::world_info::{LevelData, WorldInfoStorage};

impl WorldInfoStorage for MemoryStorage {
    fn load(&self) -> BoxFuture<'_, Result<LevelData, StorageError>> {
        Box::pin(async move {
            let guard = self.world_info.read().await;
            guard.clone().ok_or_else(|| StorageError::NotFound {
                message: "world info not stored".to_string(),
            })
        })
    }

    fn save<'a>(&'a self, data: &'a LevelData) -> BoxFuture<'a, Result<(), StorageError>> {
        Box::pin(async move {
            *self.world_info.write().await = Some(data.clone());
            Ok(())
        })
    }
}
