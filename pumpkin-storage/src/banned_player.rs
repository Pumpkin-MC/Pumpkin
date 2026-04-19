use async_trait::async_trait;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::banlist::BannedPlayerEntry;
use crate::error::StorageError;

/// Persistent storage for banned players (vanilla's `banned-players.json`).
#[async_trait]
pub trait BannedPlayerStorage: Send + Sync {
    /// Bans `uuid` (overwriting any prior ban for the same UUID).
    async fn ban(
        &self,
        uuid: Uuid,
        name: &str,
        source: String,
        expires: Option<OffsetDateTime>,
        reason: String,
    ) -> Result<(), StorageError>;

    /// Removes any ban on `uuid`. No-op if not banned.
    async fn unban(&self, uuid: Uuid) -> Result<(), StorageError>;

    /// Returns `true` when `uuid` has a ban that has not expired.
    async fn is_banned(&self, uuid: Uuid) -> Result<bool, StorageError>;

    /// Returns the active ban entry for `uuid`, if any.
    async fn get(&self, uuid: Uuid) -> Result<Option<BannedPlayerEntry>, StorageError>;

    /// Returns all currently active ban entries.
    async fn list(&self) -> Result<Vec<BannedPlayerEntry>, StorageError>;
}
