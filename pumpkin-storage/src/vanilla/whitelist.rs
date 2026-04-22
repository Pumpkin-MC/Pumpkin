//! Vanilla-compatible `whitelist.json` under the server data directory.
//! Supports in-place reload via the `/whitelist reload` command.

use std::path::PathBuf;

use async_trait::async_trait;
use pumpkin_config::whitelist::WhitelistEntry;
use uuid::Uuid;

use crate::error::StorageError;
use crate::vanilla::VanillaStorage;
use crate::vanilla::json_list::{load_json_list, save_json_list};
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
            *guard = Some(load_json_list(&self.whitelist_path()).await?);
        }
        Ok(guard)
    }
}

#[async_trait]
impl WhitelistStorage for VanillaStorage {
    async fn add(&self, uuid: Uuid, name: &str) -> Result<(), StorageError> {
        let mut guard = self.whitelist_load_locked().await?;
        let entries = guard.as_mut().expect("loaded");
        entries.retain(|e| e.uuid != uuid);
        entries.push(WhitelistEntry::new(uuid, name.to_string()));
        let snapshot = entries.clone();
        drop(guard);
        save_json_list(&self.whitelist_path(), &snapshot).await
    }

    async fn remove(&self, uuid: Uuid) -> Result<(), StorageError> {
        let mut guard = self.whitelist_load_locked().await?;
        let entries = guard.as_mut().expect("loaded");
        let before = entries.len();
        entries.retain(|e| e.uuid != uuid);
        if entries.len() == before {
            return Ok(());
        }
        let snapshot = entries.clone();
        drop(guard);
        save_json_list(&self.whitelist_path(), &snapshot).await
    }

    async fn is_whitelisted(&self, uuid: Uuid) -> Result<bool, StorageError> {
        let guard = self.whitelist_load_locked().await?;
        Ok(guard
            .as_ref()
            .expect("loaded")
            .iter()
            .any(|e| e.uuid == uuid))
    }

    async fn get(&self, uuid: Uuid) -> Result<Option<WhitelistEntry>, StorageError> {
        let guard = self.whitelist_load_locked().await?;
        Ok(guard
            .as_ref()
            .expect("loaded")
            .iter()
            .find(|e| e.uuid == uuid)
            .cloned())
    }

    async fn list(&self) -> Result<Vec<WhitelistEntry>, StorageError> {
        let guard = self.whitelist_load_locked().await?;
        Ok(guard.as_ref().expect("loaded").clone())
    }

    async fn reload(&self) -> Result<(), StorageError> {
        *self.whitelist.write().await = None;
        Ok(())
    }
}
