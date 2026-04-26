//! In-memory [`BannedPlayerStorage`] implementation. Stores entries in a
//! `Vec<BannedPlayerEntry>` behind a `RwLock`; filters expired bans at read
//! time.

use time::OffsetDateTime;
use uuid::Uuid;

use crate::BoxFuture;
use crate::banlist::BannedPlayerEntry;
use crate::banned_player::BannedPlayerStorage;
use crate::error::StorageError;
use crate::memory::MemoryStorage;

fn is_active(entry: &BannedPlayerEntry) -> bool {
    entry
        .expires
        .is_none_or(|expires| expires >= OffsetDateTime::now_utc())
}

impl BannedPlayerStorage for MemoryStorage {
    fn ban<'a>(
        &'a self,
        uuid: Uuid,
        name: &'a str,
        source: String,
        expires: Option<OffsetDateTime>,
        reason: String,
    ) -> BoxFuture<'a, Result<(), StorageError>> {
        Box::pin(async move {
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
        })
    }

    fn unban(&self, uuid: Uuid) -> BoxFuture<'_, Result<(), StorageError>> {
        Box::pin(async move {
            self.banned_players.write().await.retain(|e| e.uuid != uuid);
            Ok(())
        })
    }

    fn is_banned(&self, uuid: Uuid) -> BoxFuture<'_, Result<bool, StorageError>> {
        Box::pin(async move { Ok(BannedPlayerStorage::get(self, uuid).await?.is_some()) })
    }

    fn get(&self, uuid: Uuid) -> BoxFuture<'_, Result<Option<BannedPlayerEntry>, StorageError>> {
        Box::pin(async move {
            Ok(self
                .banned_players
                .read()
                .await
                .iter()
                .find(|e| e.uuid == uuid && is_active(e))
                .cloned())
        })
    }

    fn list(&self) -> BoxFuture<'_, Result<Vec<BannedPlayerEntry>, StorageError>> {
        Box::pin(async move {
            Ok(self
                .banned_players
                .read()
                .await
                .iter()
                .filter(|e| is_active(e))
                .cloned()
                .collect())
        })
    }
}
