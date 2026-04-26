//! In-memory [`WhitelistStorage`] backed by `Vec<WhitelistEntry>`. `reload`
//! is a no-op since there is no underlying source.

use pumpkin_config::whitelist::WhitelistEntry;
use uuid::Uuid;

use crate::BoxFuture;
use crate::error::StorageError;
use crate::memory::MemoryStorage;
use crate::whitelist::WhitelistStorage;

impl WhitelistStorage for MemoryStorage {
    fn add<'a>(&'a self, uuid: Uuid, name: &'a str) -> BoxFuture<'a, Result<(), StorageError>> {
        Box::pin(async move {
            let mut guard = self.whitelist.write().await;
            guard.retain(|e| e.uuid != uuid);
            guard.push(WhitelistEntry::new(uuid, name.to_string()));
            Ok(())
        })
    }

    fn remove(&self, uuid: Uuid) -> BoxFuture<'_, Result<(), StorageError>> {
        Box::pin(async move {
            self.whitelist.write().await.retain(|e| e.uuid != uuid);
            Ok(())
        })
    }

    fn is_whitelisted(&self, uuid: Uuid) -> BoxFuture<'_, Result<bool, StorageError>> {
        Box::pin(async move { Ok(self.whitelist.read().await.iter().any(|e| e.uuid == uuid)) })
    }

    fn get(&self, uuid: Uuid) -> BoxFuture<'_, Result<Option<WhitelistEntry>, StorageError>> {
        Box::pin(async move {
            Ok(self
                .whitelist
                .read()
                .await
                .iter()
                .find(|e| e.uuid == uuid)
                .cloned())
        })
    }

    fn list(&self) -> BoxFuture<'_, Result<Vec<WhitelistEntry>, StorageError>> {
        Box::pin(async move { Ok(self.whitelist.read().await.clone()) })
    }

    fn reload(&self) -> BoxFuture<'_, Result<(), StorageError>> {
        Box::pin(async { Ok(()) })
    }
}
