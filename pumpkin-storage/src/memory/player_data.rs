//! In-memory [`PlayerDataStorage`] — `HashMap<Uuid, PNbtCompound>`. Save
//! clones the input; load clones out.

use async_trait::async_trait;
use pumpkin_nbt::pnbt::PNbtCompound;
use uuid::Uuid;

use crate::error::StorageError;
use crate::memory::MemoryStorage;
use crate::player_data::PlayerDataStorage;

#[async_trait]
impl PlayerDataStorage for MemoryStorage {
    async fn load(&self, uuid: Uuid) -> Result<PNbtCompound, StorageError> {
        self.player_data
            .read()
            .await
            .get(&uuid)
            .cloned()
            .ok_or_else(|| StorageError::NotFound {
                message: format!("no player data for {uuid}"),
            })
    }

    async fn save(&self, uuid: Uuid, data: &PNbtCompound) -> Result<(), StorageError> {
        self.player_data.write().await.insert(uuid, data.clone());
        Ok(())
    }

    async fn list(&self) -> Result<Vec<Uuid>, StorageError> {
        Ok(self.player_data.read().await.keys().copied().collect())
    }
}
