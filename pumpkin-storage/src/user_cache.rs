//! Public entry type and trait for the user cache (`usercache.json`).

use async_trait::async_trait;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::error::StorageError;

/// MRU bookkeeping and on-disk storage is an implementation detail of the
/// `UserCacheStorage` impl; callers only see the fields defined here.
#[derive(Debug, Clone)]
pub struct UserCacheEntry {
    pub uuid: Uuid,
    pub name: String,
    pub expiration_date: OffsetDateTime,
}

/// Cache of UUID <-> name pairs for players who have connected or been
/// resolved previously. Entries expire after a while so stale names don't
/// stick around forever.
#[async_trait]
pub trait UserCacheStorage: Send + Sync {
    /// Record or refresh the UUID/name pair.
    async fn upsert(&self, uuid: Uuid, name: &str) -> Result<(), StorageError>;

    async fn get_by_uuid(&self, uuid: Uuid) -> Result<Option<UserCacheEntry>, StorageError>;

    async fn get_by_name(&self, name: &str) -> Result<Option<UserCacheEntry>, StorageError>;
}
