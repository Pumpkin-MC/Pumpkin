use std::net::IpAddr;

use async_trait::async_trait;
use time::OffsetDateTime;

use crate::banlist::BannedIpEntry;
use crate::banned_ip::BannedIpStorage;
use crate::error::StorageError;
use crate::memory::MemoryStorage;

fn is_active(entry: &BannedIpEntry) -> bool {
    entry
        .expires
        .is_none_or(|expires| expires >= OffsetDateTime::now_utc())
}

#[async_trait]
impl BannedIpStorage for MemoryStorage {
    async fn ban(
        &self,
        ip: IpAddr,
        source: String,
        expires: Option<OffsetDateTime>,
        reason: String,
    ) -> Result<(), StorageError> {
        let mut guard = self.banned_ips.write().await;
        guard.retain(|e| e.ip != ip);
        guard.push(BannedIpEntry::new(ip, source, expires, reason));
        Ok(())
    }

    async fn unban(&self, ip: IpAddr) -> Result<(), StorageError> {
        self.banned_ips.write().await.retain(|e| e.ip != ip);
        Ok(())
    }

    async fn is_banned(&self, ip: IpAddr) -> Result<bool, StorageError> {
        Ok(self.get(ip).await?.is_some())
    }

    async fn get(&self, ip: IpAddr) -> Result<Option<BannedIpEntry>, StorageError> {
        Ok(self
            .banned_ips
            .read()
            .await
            .iter()
            .find(|e| e.ip == ip && is_active(e))
            .cloned())
    }

    async fn list(&self) -> Result<Vec<BannedIpEntry>, StorageError> {
        Ok(self
            .banned_ips
            .read()
            .await
            .iter()
            .filter(|e| is_active(e))
            .cloned()
            .collect())
    }
}
