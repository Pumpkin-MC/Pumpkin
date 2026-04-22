//! In-memory [`OpStorage`] backed by `Vec<Op>` under a `RwLock`.

use async_trait::async_trait;
use pumpkin_config::op::Op;
use pumpkin_util::permission::PermissionLvl;
use uuid::Uuid;

use crate::error::StorageError;
use crate::memory::MemoryStorage;
use crate::op::OpStorage;

#[async_trait]
impl OpStorage for MemoryStorage {
    async fn op(
        &self,
        uuid: Uuid,
        name: &str,
        level: PermissionLvl,
        bypasses_player_limit: bool,
    ) -> Result<(), StorageError> {
        let mut guard = self.ops.write().await;
        guard.retain(|e| e.uuid != uuid);
        guard.push(Op::new(uuid, name.to_string(), level, bypasses_player_limit));
        Ok(())
    }

    async fn deop(&self, uuid: Uuid) -> Result<(), StorageError> {
        self.ops.write().await.retain(|e| e.uuid != uuid);
        Ok(())
    }

    async fn is_op(&self, uuid: Uuid) -> Result<bool, StorageError> {
        Ok(self.get(uuid).await?.is_some())
    }

    async fn get(&self, uuid: Uuid) -> Result<Option<Op>, StorageError> {
        Ok(self
            .ops
            .read()
            .await
            .iter()
            .find(|e| e.uuid == uuid)
            .cloned())
    }

    async fn list(&self) -> Result<Vec<Op>, StorageError> {
        Ok(self.ops.read().await.clone())
    }
}
