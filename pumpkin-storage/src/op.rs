use pumpkin_config::op::Op;
use pumpkin_util::permission::PermissionLvl;
use uuid::Uuid;

use crate::BoxFuture;
use crate::error::StorageError;

/// Persistent storage for server operators (vanilla's `ops.json`).
pub trait OpStorage: Send + Sync {
    /// Grants op status to `uuid` at the given level. Replaces any existing entry.
    fn op<'a>(
        &'a self,
        uuid: Uuid,
        name: &'a str,
        level: PermissionLvl,
        bypasses_player_limit: bool,
    ) -> BoxFuture<'a, Result<(), StorageError>>;

    /// Removes op status from `uuid`. No-op if not opped.
    fn deop(&self, uuid: Uuid) -> BoxFuture<'_, Result<(), StorageError>>;

    fn is_op(&self, uuid: Uuid) -> BoxFuture<'_, Result<bool, StorageError>>;

    fn get(&self, uuid: Uuid) -> BoxFuture<'_, Result<Option<Op>, StorageError>>;

    fn list(&self) -> BoxFuture<'_, Result<Vec<Op>, StorageError>>;
}
