use std::path::PathBuf;

use async_trait::async_trait;
use time::OffsetDateTime;
use tokio::fs;
use uuid::Uuid;

use crate::banlist::BannedPlayerEntry;
use crate::banned_player::BannedPlayerStorage;
use crate::error::StorageError;
use crate::vanilla::VanillaStorage;

const BANNED_PLAYERS_FILE: &str = "banned-players.json";

impl VanillaStorage {
    fn banned_players_path(&self) -> PathBuf {
        self.server_data_dir().join(BANNED_PLAYERS_FILE)
    }

    async fn banned_players_load_locked(
        &self,
    ) -> Result<
        tokio::sync::RwLockWriteGuard<'_, Option<Vec<BannedPlayerEntry>>>,
        StorageError,
    > {
        let mut guard = self.banned_players.write().await;
        if guard.is_none() {
            *guard = Some(load_banned_players(&self.banned_players_path()).await?);
        }
        Ok(guard)
    }

    async fn banned_players_flush(
        &self,
        entries: &[BannedPlayerEntry],
    ) -> Result<(), StorageError> {
        save_banned_players(&self.banned_players_path(), entries).await
    }
}

async fn load_banned_players(path: &std::path::Path) -> Result<Vec<BannedPlayerEntry>, StorageError> {
    match fs::read_to_string(path).await {
        Ok(content) => serde_json::from_str::<Vec<BannedPlayerEntry>>(&content)
            .map_err(|e| StorageError::Deserialize(e.to_string())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(e) => Err(StorageError::io_at(path, e)),
    }
}

async fn save_banned_players(
    path: &std::path::Path,
    entries: &[BannedPlayerEntry],
) -> Result<(), StorageError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .await
            .map_err(|e| StorageError::io_at(parent, e))?;
    }
    let content = serde_json::to_string_pretty(entries)
        .map_err(|e| StorageError::Serialize(e.to_string()))?;
    fs::write(path, content)
        .await
        .map_err(|e| StorageError::io_at(path, e))?;
    Ok(())
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
        self.banned_players_flush(&snapshot).await
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
        self.banned_players_flush(&snapshot).await
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
