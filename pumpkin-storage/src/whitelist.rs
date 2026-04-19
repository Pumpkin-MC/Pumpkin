use async_trait::async_trait;
use pumpkin_config::whitelist::WhitelistEntry;
use uuid::Uuid;

use crate::error::StorageError;

/// Persistent storage for whitelisted players (vanilla's `whitelist.json`).
#[async_trait]
pub trait WhitelistStorage: Send + Sync {
    async fn add(&self, uuid: Uuid, name: &str) -> Result<(), StorageError>;

    async fn remove(&self, uuid: Uuid) -> Result<(), StorageError>;

    async fn is_whitelisted(&self, uuid: Uuid) -> Result<bool, StorageError>;

    async fn list(&self) -> Result<Vec<WhitelistEntry>, StorageError>;
}
