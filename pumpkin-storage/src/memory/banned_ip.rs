//! In-memory [`BannedIpStorage`] implementation. Stores entries in a
//! `Vec<BannedIpEntry>` behind a `RwLock`; filters expired bans at read time.

use std::net::IpAddr;

use time::OffsetDateTime;

use crate::BoxFuture;
use crate::banlist::BannedIpEntry;
use crate::banned_ip::BannedIpStorage;
use crate::error::StorageError;
use crate::memory::MemoryStorage;

fn is_active(entry: &BannedIpEntry) -> bool {
    entry
        .expires
        .is_none_or(|expires| expires >= OffsetDateTime::now_utc())
}

impl BannedIpStorage for MemoryStorage {
    fn ban(
        &self,
        ip: IpAddr,
        source: String,
        expires: Option<OffsetDateTime>,
        reason: String,
    ) -> BoxFuture<'_, Result<(), StorageError>> {
        Box::pin(async move {
            let mut guard = self.banned_ips.write().await;
            guard.retain(|e| e.ip != ip);
            guard.push(BannedIpEntry::new(ip, source, expires, reason));
            Ok(())
        })
    }

    fn unban(&self, ip: IpAddr) -> BoxFuture<'_, Result<(), StorageError>> {
        Box::pin(async move {
            self.banned_ips.write().await.retain(|e| e.ip != ip);
            Ok(())
        })
    }

    fn is_banned(&self, ip: IpAddr) -> BoxFuture<'_, Result<bool, StorageError>> {
        Box::pin(async move { Ok(BannedIpStorage::get(self, ip).await?.is_some()) })
    }

    fn get(&self, ip: IpAddr) -> BoxFuture<'_, Result<Option<BannedIpEntry>, StorageError>> {
        Box::pin(async move {
            Ok(self
                .banned_ips
                .read()
                .await
                .iter()
                .find(|e| e.ip == ip && is_active(e))
                .cloned())
        })
    }

    fn list(&self) -> BoxFuture<'_, Result<Vec<BannedIpEntry>, StorageError>> {
        Box::pin(async move {
            Ok(self
                .banned_ips
                .read()
                .await
                .iter()
                .filter(|e| is_active(e))
                .cloned()
                .collect())
        })
    }
}
