//! Vanilla-compatible `ops.json` under the server data directory.

use std::path::PathBuf;

use async_trait::async_trait;
use pumpkin_config::op::Op;
use pumpkin_util::permission::PermissionLvl;
use uuid::Uuid;

use crate::error::StorageError;
use crate::op::OpStorage;
use crate::vanilla::VanillaStorage;
use crate::vanilla::json_list::{load_json_list, save_json_list};

const OPS_FILE: &str = "ops.json";

impl VanillaStorage {
    fn ops_path(&self) -> PathBuf {
        self.server_data_dir().join(OPS_FILE)
    }

    async fn ops_load_locked(
        &self,
    ) -> Result<tokio::sync::RwLockWriteGuard<'_, Option<Vec<Op>>>, StorageError> {
        let mut guard = self.ops.write().await;
        if guard.is_none() {
            *guard = Some(load_json_list(&self.ops_path()).await?);
        }
        Ok(guard)
    }
}

#[async_trait]
impl OpStorage for VanillaStorage {
    async fn op(
        &self,
        uuid: Uuid,
        name: &str,
        level: PermissionLvl,
        bypasses_player_limit: bool,
    ) -> Result<(), StorageError> {
        let mut guard = self.ops_load_locked().await?;
        let entries = guard.as_mut().expect("loaded");
        entries.retain(|e| e.uuid != uuid);
        entries.push(Op::new(
            uuid,
            name.to_string(),
            level,
            bypasses_player_limit,
        ));
        let snapshot = entries.clone();
        drop(guard);
        save_json_list(&self.ops_path(), &snapshot).await
    }

    async fn deop(&self, uuid: Uuid) -> Result<(), StorageError> {
        let mut guard = self.ops_load_locked().await?;
        let entries = guard.as_mut().expect("loaded");
        let before = entries.len();
        entries.retain(|e| e.uuid != uuid);
        if entries.len() == before {
            return Ok(());
        }
        let snapshot = entries.clone();
        drop(guard);
        save_json_list(&self.ops_path(), &snapshot).await
    }

    async fn is_op(&self, uuid: Uuid) -> Result<bool, StorageError> {
        Ok(self.get(uuid).await?.is_some())
    }

    async fn get(&self, uuid: Uuid) -> Result<Option<Op>, StorageError> {
        let guard = self.ops_load_locked().await?;
        Ok(guard
            .as_ref()
            .expect("loaded")
            .iter()
            .find(|e| e.uuid == uuid)
            .cloned())
    }

    async fn list(&self) -> Result<Vec<Op>, StorageError> {
        let guard = self.ops_load_locked().await?;
        Ok(guard.as_ref().expect("loaded").clone())
    }
}
