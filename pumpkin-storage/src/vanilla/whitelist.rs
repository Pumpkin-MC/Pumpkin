use std::path::PathBuf;

use async_trait::async_trait;
use pumpkin_config::whitelist::WhitelistEntry;
use tokio::fs;
use uuid::Uuid;

use crate::error::StorageError;
use crate::vanilla::VanillaStorage;
use crate::whitelist::WhitelistStorage;

const WHITELIST_FILE: &str = "whitelist.json";

impl VanillaStorage {
    fn whitelist_path(&self) -> PathBuf {
        self.server_data_dir().join(WHITELIST_FILE)
    }

    async fn whitelist_load_locked(
        &self,
    ) -> Result<tokio::sync::RwLockWriteGuard<'_, Option<Vec<WhitelistEntry>>>, StorageError> {
        let mut guard = self.whitelist.write().await;
        if guard.is_none() {
            *guard = Some(load_whitelist(&self.whitelist_path()).await?);
        }
        Ok(guard)
    }
}

async fn load_whitelist(path: &std::path::Path) -> Result<Vec<WhitelistEntry>, StorageError> {
    match fs::read_to_string(path).await {
        Ok(content) => serde_json::from_str::<Vec<WhitelistEntry>>(&content)
            .map_err(|e| StorageError::Deserialize(e.to_string())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(e) => Err(StorageError::io_at(path, e)),
    }
}

async fn save_whitelist(
    path: &std::path::Path,
    entries: &[WhitelistEntry],
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

#[async_trait]
impl WhitelistStorage for VanillaStorage {
    async fn add(&self, uuid: Uuid, name: &str) -> Result<(), StorageError> {
        let mut guard = self.whitelist_load_locked().await?;
        let entries = guard.as_mut().expect("loaded");
        entries.retain(|e| e.uuid != uuid);
        entries.push(WhitelistEntry::new(uuid, name.to_string()));
        let snapshot = clone_entries(entries);
        drop(guard);
        save_whitelist(&self.whitelist_path(), &snapshot).await
    }

    async fn remove(&self, uuid: Uuid) -> Result<(), StorageError> {
        let mut guard = self.whitelist_load_locked().await?;
        let entries = guard.as_mut().expect("loaded");
        let before = entries.len();
        entries.retain(|e| e.uuid != uuid);
        if entries.len() == before {
            return Ok(());
        }
        let snapshot = clone_entries(entries);
        drop(guard);
        save_whitelist(&self.whitelist_path(), &snapshot).await
    }

    async fn is_whitelisted(&self, uuid: Uuid) -> Result<bool, StorageError> {
        let guard = self.whitelist_load_locked().await?;
        Ok(guard
            .as_ref()
            .expect("loaded")
            .iter()
            .any(|e| e.uuid == uuid))
    }

    async fn list(&self) -> Result<Vec<WhitelistEntry>, StorageError> {
        let guard = self.whitelist_load_locked().await?;
        Ok(clone_entries(guard.as_ref().expect("loaded")))
    }

    async fn reload(&self) -> Result<(), StorageError> {
        *self.whitelist.write().await = None;
        Ok(())
    }
}

fn clone_entries(entries: &[WhitelistEntry]) -> Vec<WhitelistEntry> {
    entries
        .iter()
        .map(|e| WhitelistEntry::new(e.uuid, e.name.clone()))
        .collect()
}
