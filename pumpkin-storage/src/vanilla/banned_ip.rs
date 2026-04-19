use std::net::IpAddr;
use std::path::PathBuf;

use async_trait::async_trait;
use time::OffsetDateTime;
use tokio::fs;

use crate::banlist::BannedIpEntry;
use crate::banned_ip::BannedIpStorage;
use crate::error::StorageError;
use crate::vanilla::VanillaStorage;

const BANNED_IPS_FILE: &str = "banned-ips.json";

impl VanillaStorage {
    fn banned_ips_path(&self) -> PathBuf {
        self.server_data_dir().join(BANNED_IPS_FILE)
    }

    async fn banned_ips_load_locked(
        &self,
    ) -> Result<tokio::sync::RwLockWriteGuard<'_, Option<Vec<BannedIpEntry>>>, StorageError> {
        let mut guard = self.banned_ips.write().await;
        if guard.is_none() {
            *guard = Some(load_banned_ips(&self.banned_ips_path()).await?);
        }
        Ok(guard)
    }
}

async fn load_banned_ips(path: &std::path::Path) -> Result<Vec<BannedIpEntry>, StorageError> {
    match fs::read_to_string(path).await {
        Ok(content) => serde_json::from_str::<Vec<BannedIpEntry>>(&content)
            .map_err(|e| StorageError::Deserialize(e.to_string())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(e) => Err(StorageError::io_at(path, e)),
    }
}

async fn save_banned_ips(
    path: &std::path::Path,
    entries: &[BannedIpEntry],
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

fn is_active(entry: &BannedIpEntry) -> bool {
    entry
        .expires
        .is_none_or(|expires| expires >= OffsetDateTime::now_utc())
}

#[async_trait]
impl BannedIpStorage for VanillaStorage {
    async fn ban(
        &self,
        ip: IpAddr,
        source: String,
        expires: Option<OffsetDateTime>,
        reason: String,
    ) -> Result<(), StorageError> {
        let mut guard = self.banned_ips_load_locked().await?;
        let entries = guard.as_mut().expect("loaded");
        entries.retain(|e| e.ip != ip);
        entries.push(BannedIpEntry::new(ip, source, expires, reason));
        let snapshot = entries.clone();
        drop(guard);
        save_banned_ips(&self.banned_ips_path(), &snapshot).await
    }

    async fn unban(&self, ip: IpAddr) -> Result<(), StorageError> {
        let mut guard = self.banned_ips_load_locked().await?;
        let entries = guard.as_mut().expect("loaded");
        let before = entries.len();
        entries.retain(|e| e.ip != ip);
        if entries.len() == before {
            return Ok(());
        }
        let snapshot = entries.clone();
        drop(guard);
        save_banned_ips(&self.banned_ips_path(), &snapshot).await
    }

    async fn is_banned(&self, ip: IpAddr) -> Result<bool, StorageError> {
        Ok(self.get(ip).await?.is_some())
    }

    async fn get(&self, ip: IpAddr) -> Result<Option<BannedIpEntry>, StorageError> {
        let guard = self.banned_ips_load_locked().await?;
        Ok(guard
            .as_ref()
            .expect("loaded")
            .iter()
            .find(|e| e.ip == ip && is_active(e))
            .cloned())
    }

    async fn list(&self) -> Result<Vec<BannedIpEntry>, StorageError> {
        let guard = self.banned_ips_load_locked().await?;
        Ok(guard
            .as_ref()
            .expect("loaded")
            .iter()
            .filter(|e| is_active(e))
            .cloned()
            .collect())
    }
}
