//! Vanilla-compatible `whitelist.json` under the server data directory.
//! Supports in-place reload via the `/whitelist reload` command.

use std::path::PathBuf;

use pumpkin_config::whitelist::WhitelistEntry;
use uuid::Uuid;

use crate::BoxFuture;
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

impl WhitelistStorage for VanillaStorage {
    fn add<'a>(&'a self, uuid: Uuid, name: &'a str) -> BoxFuture<'a, Result<(), StorageError>> {
        Box::pin(async move {
            let mut guard = self.whitelist_load_locked().await?;
            let entries = guard.as_mut().expect("loaded");
            entries.retain(|e| e.uuid != uuid);
            entries.push(WhitelistEntry::new(uuid, name.to_string()));
            let snapshot = entries.clone();
            drop(guard);
            save_json_list(&self.whitelist_path(), &snapshot).await
        })
    }

    fn remove(&self, uuid: Uuid) -> BoxFuture<'_, Result<(), StorageError>> {
        Box::pin(async move {
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
        })
    }

    fn is_whitelisted(&self, uuid: Uuid) -> BoxFuture<'_, Result<bool, StorageError>> {
        Box::pin(async move {
            let guard = self.whitelist_load_locked().await?;
            Ok(guard
                .as_ref()
                .expect("loaded")
                .iter()
                .any(|e| e.uuid == uuid))
        })
    }

    fn get(&self, uuid: Uuid) -> BoxFuture<'_, Result<Option<WhitelistEntry>, StorageError>> {
        Box::pin(async move {
            let guard = self.whitelist_load_locked().await?;
            Ok(guard
                .as_ref()
                .expect("loaded")
                .iter()
                .find(|e| e.uuid == uuid)
                .cloned())
        })
    }

    fn list(&self) -> BoxFuture<'_, Result<Vec<WhitelistEntry>, StorageError>> {
        Box::pin(async move {
            let guard = self.whitelist_load_locked().await?;
            Ok(guard.as_ref().expect("loaded").clone())
        })
    }

    fn reload(&self) -> BoxFuture<'_, Result<(), StorageError>> {
        Box::pin(async move {
            *self.whitelist.write().await = None;
            Ok(())
        })
    }
}
