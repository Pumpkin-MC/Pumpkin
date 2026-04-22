//! Vanilla-compatible `banned-players.json` under the server data directory.
//! Expired bans are filtered at read time but remain on disk until the next
//! write.

use std::path::PathBuf;

use async_trait::async_trait;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::banlist::BannedPlayerEntry;
use crate::banned_player::BannedPlayerStorage;
use crate::error::StorageError;
use crate::vanilla::VanillaStorage;
use crate::vanilla::json_list::{load_json_list, save_json_list};

const BANNED_PLAYERS_FILE: &str = "banned-players.json";

impl VanillaStorage {
    fn banned_players_path(&self) -> PathBuf {
        self.server_data_dir().join(BANNED_PLAYERS_FILE)
    }

    async fn banned_players_load_locked(
        &self,
    ) -> Result<tokio::sync::RwLockWriteGuard<'_, Option<Vec<BannedPlayerEntry>>>, StorageError>
    {
        let mut guard = self.banned_players.write().await;
        if guard.is_none() {
            *guard = Some(load_json_list(&self.banned_players_path()).await?);
        }
        Ok(guard)
    }
}

fn is_active(entry: &BannedPlayerEntry) -> bool {
    entry
        .expires
        .is_none_or(|expires| expires >= OffsetDateTime::now_utc())
}

#[async_trait]
impl BannedPlayerStorage for VanillaStorage {
    async fn ban(
        &self,
        uuid: Uuid,
        name: &str,
        source: String,
        expires: Option<OffsetDateTime>,
        reason: String,
    ) -> Result<(), StorageError> {
        let mut guard = self.banned_players_load_locked().await?;
        let entries = guard.as_mut().expect("loaded");
        entries.retain(|e| e.uuid != uuid);
        entries.push(BannedPlayerEntry::new(
            uuid,
            name.to_string(),
            source,
            expires,
            reason,
        ));
        let snapshot = entries.clone();
        drop(guard);
        save_json_list(&self.banned_players_path(), &snapshot).await
    }

    async fn unban(&self, uuid: Uuid) -> Result<(), StorageError> {
        let mut guard = self.banned_players_load_locked().await?;
        let entries = guard.as_mut().expect("loaded");
        let before = entries.len();
        entries.retain(|e| e.uuid != uuid);
        if entries.len() == before {
            return Ok(());
        }
        let snapshot = entries.clone();
        drop(guard);
        save_json_list(&self.banned_players_path(), &snapshot).await
    }

    async fn is_banned(&self, uuid: Uuid) -> Result<bool, StorageError> {
        Ok(self.get(uuid).await?.is_some())
    }

    async fn get(&self, uuid: Uuid) -> Result<Option<BannedPlayerEntry>, StorageError> {
        let guard = self.banned_players_load_locked().await?;
        Ok(guard
            .as_ref()
            .expect("loaded")
            .iter()
            .find(|e| e.uuid == uuid && is_active(e))
            .cloned())
    }

    async fn list(&self) -> Result<Vec<BannedPlayerEntry>, StorageError> {
        let guard = self.banned_players_load_locked().await?;
        Ok(guard
            .as_ref()
            .expect("loaded")
            .iter()
            .filter(|e| is_active(e))
            .cloned()
            .collect())
    }
}
