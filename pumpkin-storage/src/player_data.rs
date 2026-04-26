use pumpkin_nbt::pnbt::PNbtCompound;
use uuid::Uuid;

use crate::BoxFuture;
use crate::error::StorageError;

/// Persistent storage for per-player NBT data (vanilla's `playerdata/<uuid>.dat`).
pub trait PlayerDataStorage: Send + Sync {
    /// Reads the stored NBT for `uuid`.
    ///
    /// Returns an error for which [`StorageError::is_not_found`] is `true`
    /// when no data has been stored for that player.
    fn load(&self, uuid: Uuid) -> BoxFuture<'_, Result<PNbtCompound, StorageError>>;

    /// Persists `data` as the current NBT for `uuid`, overwriting any prior value.
    fn save<'a>(
        &'a self,
        uuid: Uuid,
        data: &'a PNbtCompound,
    ) -> BoxFuture<'a, Result<(), StorageError>>;

    /// Lists every UUID for which data is currently stored.
    ///
    /// Order is implementation-defined.
    fn list(&self) -> BoxFuture<'_, Result<Vec<Uuid>, StorageError>>;
}
