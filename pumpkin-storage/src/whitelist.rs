use pumpkin_config::whitelist::WhitelistEntry;
use uuid::Uuid;

use crate::BoxFuture;
use crate::error::StorageError;

/// Persistent storage for whitelisted players (vanilla's `whitelist.json`).
pub trait WhitelistStorage: Send + Sync {
    fn add<'a>(&'a self, uuid: Uuid, name: &'a str) -> BoxFuture<'a, Result<(), StorageError>>;

    fn remove(&self, uuid: Uuid) -> BoxFuture<'_, Result<(), StorageError>>;

    fn is_whitelisted(&self, uuid: Uuid) -> BoxFuture<'_, Result<bool, StorageError>>;

    /// Returns the whitelist entry for `uuid`, if any.
    fn get(&self, uuid: Uuid) -> BoxFuture<'_, Result<Option<WhitelistEntry>, StorageError>>;

    fn list(&self) -> BoxFuture<'_, Result<Vec<WhitelistEntry>, StorageError>>;

    /// Drops any in-memory cache so the next read re-reads the underlying
    /// source. Used by the `/whitelist reload` command.
    fn reload(&self) -> BoxFuture<'_, Result<(), StorageError>>;
}
