use async_trait::async_trait;
use pumpkin_config::op::Op;
use pumpkin_util::permission::PermissionLvl;
use uuid::Uuid;

use crate::error::StorageError;

/// Persistent storage for server operators (vanilla's `ops.json`).
#[async_trait]
pub trait OpStorage: Send + Sync {
    /// Grants op status to `uuid` at the given level. Replaces any existing entry.
    async fn op(
        &self,
        uuid: Uuid,
        name: &str,
        level: PermissionLvl,
        bypasses_player_limit: bool,
    ) -> Result<(), StorageError>;

    /// Removes op status from `uuid`. No-op if not opped.
    async fn deop(&self, uuid: Uuid) -> Result<(), StorageError>;

    async fn is_op(&self, uuid: Uuid) -> Result<bool, StorageError>;

    async fn get(&self, uuid: Uuid) -> Result<Option<Op>, StorageError>;

    async fn list(&self) -> Result<Vec<Op>, StorageError>;
}
