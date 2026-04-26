use time::OffsetDateTime;
use uuid::Uuid;

use crate::BoxFuture;
use crate::banlist::BannedPlayerEntry;
use crate::error::StorageError;

/// Persistent storage for banned players (vanilla's `banned-players.json`).
pub trait BannedPlayerStorage: Send + Sync {
    /// Bans `uuid` (overwriting any prior ban for the same UUID).
    fn ban<'a>(
        &'a self,
        uuid: Uuid,
        name: &'a str,
        source: String,
        expires: Option<OffsetDateTime>,
        reason: String,
    ) -> BoxFuture<'a, Result<(), StorageError>>;

    /// Removes any ban on `uuid`. No-op if not banned.
    fn unban(&self, uuid: Uuid) -> BoxFuture<'_, Result<(), StorageError>>;

    /// Returns `true` when `uuid` has a ban that has not expired.
    fn is_banned(&self, uuid: Uuid) -> BoxFuture<'_, Result<bool, StorageError>>;

    /// Returns the active ban entry for `uuid`, if any.
    fn get(&self, uuid: Uuid) -> BoxFuture<'_, Result<Option<BannedPlayerEntry>, StorageError>>;

    /// Returns all currently active ban entries.
    fn list(&self) -> BoxFuture<'_, Result<Vec<BannedPlayerEntry>, StorageError>>;
}
