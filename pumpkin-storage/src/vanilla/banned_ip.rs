//! Vanilla-compatible `banned-ips.json` under the server data directory.
//! Expired bans are filtered at read time but remain on disk until the next
//! write.

use std::net::IpAddr;
use std::path::PathBuf;

use time::OffsetDateTime;

use crate::BoxFuture;
use crate::banlist::BannedIpEntry;
use crate::banned_ip::BannedIpStorage;
use crate::error::StorageError;
use crate::vanilla::VanillaStorage;
use crate::vanilla::json_list::{load_json_list, save_json_list};

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
            *guard = Some(load_json_list(&self.banned_ips_path()).await?);
        }
        Ok(guard)
    }
}

fn is_active(entry: &BannedIpEntry) -> bool {
    entry
        .expires
        .is_none_or(|expires| expires >= OffsetDateTime::now_utc())
}

impl BannedIpStorage for VanillaStorage {
    fn ban(
        &self,
        ip: IpAddr,
        source: String,
        expires: Option<OffsetDateTime>,
        reason: String,
    ) -> BoxFuture<'_, Result<(), StorageError>> {
        Box::pin(async move {
            let mut guard = self.banned_ips_load_locked().await?;
            let entries = guard.as_mut().expect("loaded");
            entries.retain(|e| e.ip != ip);
            entries.push(BannedIpEntry::new(ip, source, expires, reason));
            let snapshot = entries.clone();
            drop(guard);
            save_json_list(&self.banned_ips_path(), &snapshot).await
        })
    }

    fn unban(&self, ip: IpAddr) -> BoxFuture<'_, Result<(), StorageError>> {
        Box::pin(async move {
            let mut guard = self.banned_ips_load_locked().await?;
            let entries = guard.as_mut().expect("loaded");
            let before = entries.len();
            entries.retain(|e| e.ip != ip);
            if entries.len() == before {
                return Ok(());
            }
            let snapshot = entries.clone();
            drop(guard);
            save_json_list(&self.banned_ips_path(), &snapshot).await
        })
    }

    fn is_banned(&self, ip: IpAddr) -> BoxFuture<'_, Result<bool, StorageError>> {
        Box::pin(async move { Ok(BannedIpStorage::get(self, ip).await?.is_some()) })
    }

    fn get(&self, ip: IpAddr) -> BoxFuture<'_, Result<Option<BannedIpEntry>, StorageError>> {
        Box::pin(async move {
            let guard = self.banned_ips_load_locked().await?;
            Ok(guard
                .as_ref()
                .expect("loaded")
                .iter()
                .find(|e| e.ip == ip && is_active(e))
                .cloned())
        })
    }

    fn list(&self) -> BoxFuture<'_, Result<Vec<BannedIpEntry>, StorageError>> {
        Box::pin(async move {
            let guard = self.banned_ips_load_locked().await?;
            Ok(guard
                .as_ref()
                .expect("loaded")
                .iter()
                .filter(|e| is_active(e))
                .cloned()
                .collect())
        })
    }
}
