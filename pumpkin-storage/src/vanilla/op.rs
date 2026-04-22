//! Vanilla-compatible `ops.json` under the server data directory.

use std::path::PathBuf;

use async_trait::async_trait;
use pumpkin_config::op::Op;
use pumpkin_util::permission::PermissionLvl;
use tokio::fs;
use uuid::Uuid;

use crate::error::StorageError;
use crate::op::OpStorage;
use crate::vanilla::VanillaStorage;

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
            *guard = Some(load_ops(&self.ops_path()).await?);
        }
        Ok(guard)
    }
}

async fn load_ops(path: &std::path::Path) -> Result<Vec<Op>, StorageError> {
    match fs::read_to_string(path).await {
        Ok(content) => serde_json::from_str::<Vec<Op>>(&content)
            .map_err(|e| StorageError::Deserialize(e.to_string())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(e) => Err(StorageError::io_at(path, e)),
    }
}

async fn save_ops(path: &std::path::Path, entries: &[Op]) -> Result<(), StorageError> {
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
        entries.push(Op::new(uuid, name.to_string(), level, bypasses_player_limit));
        let snapshot = entries.clone();
        drop(guard);
        save_ops(&self.ops_path(), &snapshot).await
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
        save_ops(&self.ops_path(), &snapshot).await
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
