//! In-memory [`OpStorage`] backed by `Vec<Op>` under a `RwLock`.

use pumpkin_config::op::Op;
use pumpkin_util::permission::PermissionLvl;
use uuid::Uuid;

use crate::BoxFuture;
use crate::error::StorageError;
use crate::memory::MemoryStorage;
use crate::op::OpStorage;

impl OpStorage for MemoryStorage {
    fn op<'a>(
        &'a self,
        uuid: Uuid,
        name: &'a str,
        level: PermissionLvl,
        bypasses_player_limit: bool,
    ) -> BoxFuture<'a, Result<(), StorageError>> {
        Box::pin(async move {
            let mut guard = self.ops.write().await;
            guard.retain(|e| e.uuid != uuid);
            guard.push(Op::new(
                uuid,
                name.to_string(),
                level,
                bypasses_player_limit,
            ));
            Ok(())
        })
    }

    fn deop(&self, uuid: Uuid) -> BoxFuture<'_, Result<(), StorageError>> {
        Box::pin(async move {
            self.ops.write().await.retain(|e| e.uuid != uuid);
            Ok(())
        })
    }

    fn is_op(&self, uuid: Uuid) -> BoxFuture<'_, Result<bool, StorageError>> {
        Box::pin(async move { Ok(OpStorage::get(self, uuid).await?.is_some()) })
    }

    fn get(&self, uuid: Uuid) -> BoxFuture<'_, Result<Option<Op>, StorageError>> {
        Box::pin(async move {
            Ok(self
                .ops
                .read()
                .await
                .iter()
                .find(|e| e.uuid == uuid)
                .cloned())
        })
    }

    fn list(&self) -> BoxFuture<'_, Result<Vec<Op>, StorageError>> {
        Box::pin(async move { Ok(self.ops.read().await.clone()) })
    }
}
