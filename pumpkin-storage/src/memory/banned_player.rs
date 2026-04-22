//! In-memory [`BannedPlayerStorage`] implementation. Stores entries in a
//! `Vec<BannedPlayerEntry>` behind a `RwLock`; filters expired bans at read
//! time.

use async_trait::async_trait;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::banlist::BannedPlayerEntry;
use crate::banned_player::BannedPlayerStorage;
use crate::error::StorageError;
use crate::memory::MemoryStorage;

fn is_active(entry: &BannedPlayerEntry) -> bool {
    entry
        .expires
        .is_none_or(|expires| expires >= OffsetDateTime::now_utc())
}

#[async_trait]
impl BannedPlayerStorage for MemoryStorage {
    async fn ban(
        &self,
        uuid: Uuid,
        name: &str,
        source: String,
        expires: Option<OffsetDateTime>,
        reason: String,
    ) -> Result<(), StorageError> {
        let mut guard = self.banned_players.write().await;
        guard.retain(|e| e.uuid != uuid);
        guard.push(BannedPlayerEntry::new(
            uuid,
            name.to_string(),
            source,
            expires,
            reason,
        ));
        Ok(())
    }

    async fn unban(&self, uuid: Uuid) -> Result<(), StorageError> {
        self.banned_players
            .write()
            .await
            .retain(|e| e.uuid != uuid);
        Ok(())
    }

    async fn is_banned(&self, uuid: Uuid) -> Result<bool, StorageError> {
        Ok(self.get(uuid).await?.is_some())
    }

    async fn get(&self, uuid: Uuid) -> Result<Option<BannedPlayerEntry>, StorageError> {
        Ok(self
            .banned_players
            .read()
            .await
            .iter()
            .find(|e| e.uuid == uuid && is_active(e))
            .cloned())
    }

    async fn list(&self) -> Result<Vec<BannedPlayerEntry>, StorageError> {
        Ok(self
            .banned_players
            .read()
            .await
            .iter()
            .filter(|e| is_active(e))
            .cloned()
            .collect())
    }
}
