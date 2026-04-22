use async_trait::async_trait;
use pumpkin_nbt::pnbt::PNbtCompound;
use uuid::Uuid;

use crate::error::StorageError;

/// Persistent storage for per-player NBT data (vanilla's `playerdata/<uuid>.dat`).
#[async_trait]
pub trait PlayerDataStorage: Send + Sync {
    /// Reads the stored NBT for `uuid`.
    ///
    /// Returns an error for which [`StorageError::is_not_found`] is `true`
    /// when no data has been stored for that player.
    async fn load(&self, uuid: Uuid) -> Result<PNbtCompound, StorageError>;

    /// Persists `data` as the current NBT for `uuid`, overwriting any prior value.
    async fn save(&self, uuid: Uuid, data: &PNbtCompound) -> Result<(), StorageError>;

    /// Lists every UUID for which data is currently stored.
    ///
    /// Order is implementation-defined.
    async fn list(&self) -> Result<Vec<Uuid>, StorageError>;
}
