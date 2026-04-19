use std::net::IpAddr;

use async_trait::async_trait;
use time::OffsetDateTime;

use crate::banlist::BannedIpEntry;
use crate::error::StorageError;

/// Persistent storage for banned IPs (vanilla's `banned-ips.json`).
#[async_trait]
pub trait BannedIpStorage: Send + Sync {
    async fn ban(
        &self,
        ip: IpAddr,
        source: String,
        expires: Option<OffsetDateTime>,
        reason: String,
    ) -> Result<(), StorageError>;

    async fn unban(&self, ip: IpAddr) -> Result<(), StorageError>;

    async fn is_banned(&self, ip: IpAddr) -> Result<bool, StorageError>;

    async fn get(&self, ip: IpAddr) -> Result<Option<BannedIpEntry>, StorageError>;

    async fn list(&self) -> Result<Vec<BannedIpEntry>, StorageError>;
}
