use std::net::IpAddr;

use time::OffsetDateTime;

use crate::BoxFuture;
use crate::banlist::BannedIpEntry;
use crate::error::StorageError;

/// Persistent storage for banned IPs (vanilla's `banned-ips.json`).
pub trait BannedIpStorage: Send + Sync {
    fn ban(
        &self,
        ip: IpAddr,
        source: String,
        expires: Option<OffsetDateTime>,
        reason: String,
    ) -> BoxFuture<'_, Result<(), StorageError>>;

    fn unban(&self, ip: IpAddr) -> BoxFuture<'_, Result<(), StorageError>>;

    fn is_banned(&self, ip: IpAddr) -> BoxFuture<'_, Result<bool, StorageError>>;

    fn get(&self, ip: IpAddr) -> BoxFuture<'_, Result<Option<BannedIpEntry>, StorageError>>;

    fn list(&self) -> BoxFuture<'_, Result<Vec<BannedIpEntry>, StorageError>>;
}
