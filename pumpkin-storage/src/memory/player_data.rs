//! In-memory [`PlayerDataStorage`] — `HashMap<Uuid, PNbtCompound>`. Save
//! clones the input; load clones out.

use pumpkin_nbt::pnbt::PNbtCompound;
use uuid::Uuid;

use crate::BoxFuture;
use crate::error::StorageError;
use crate::memory::MemoryStorage;
use crate::player_data::PlayerDataStorage;

impl PlayerDataStorage for MemoryStorage {
    fn load(&self, uuid: Uuid) -> BoxFuture<'_, Result<PNbtCompound, StorageError>> {
        Box::pin(async move {
            self.player_data
                .read()
                .await
                .get(&uuid)
                .cloned()
                .ok_or_else(|| StorageError::NotFound {
                    message: format!("no player data for {uuid}"),
                })
        })
    }

    fn save<'a>(
        &'a self,
        uuid: Uuid,
        data: &'a PNbtCompound,
    ) -> BoxFuture<'a, Result<(), StorageError>> {
        Box::pin(async move {
            self.player_data.write().await.insert(uuid, data.clone());
            Ok(())
        })
    }

    fn list(&self) -> BoxFuture<'_, Result<Vec<Uuid>, StorageError>> {
        Box::pin(async move { Ok(self.player_data.read().await.keys().copied().collect()) })
    }
}
