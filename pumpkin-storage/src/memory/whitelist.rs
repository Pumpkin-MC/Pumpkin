//! In-memory [`WhitelistStorage`] backed by `Vec<WhitelistEntry>`. `reload`
//! is a no-op since there is no underlying source.

use async_trait::async_trait;
use pumpkin_config::whitelist::WhitelistEntry;
use uuid::Uuid;

use crate::error::StorageError;
use crate::memory::MemoryStorage;
use crate::whitelist::WhitelistStorage;

#[async_trait]
impl WhitelistStorage for MemoryStorage {
    async fn add(&self, uuid: Uuid, name: &str) -> Result<(), StorageError> {
        let mut guard = self.whitelist.write().await;
        guard.retain(|e| e.uuid != uuid);
        guard.push(WhitelistEntry::new(uuid, name.to_string()));
        Ok(())
    }

    async fn remove(&self, uuid: Uuid) -> Result<(), StorageError> {
        self.whitelist.write().await.retain(|e| e.uuid != uuid);
        Ok(())
    }

    async fn is_whitelisted(&self, uuid: Uuid) -> Result<bool, StorageError> {
        Ok(self.whitelist.read().await.iter().any(|e| e.uuid == uuid))
    }

    async fn get(&self, uuid: Uuid) -> Result<Option<WhitelistEntry>, StorageError> {
        Ok(self
            .whitelist
            .read()
            .await
            .iter()
            .find(|e| e.uuid == uuid)
            .cloned())
    }

    async fn list(&self) -> Result<Vec<WhitelistEntry>, StorageError> {
        Ok(self.whitelist.read().await.clone())
    }

    async fn reload(&self) -> Result<(), StorageError> {
        Ok(())
    }
}
