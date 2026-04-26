//! Public entry type and trait for the user cache (`usercache.json`).

use time::OffsetDateTime;
use uuid::Uuid;

use crate::BoxFuture;
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
pub trait UserCacheStorage: Send + Sync {
    /// Record or refresh the UUID/name pair.
    fn upsert<'a>(&'a self, uuid: Uuid, name: &'a str) -> BoxFuture<'a, Result<(), StorageError>>;

    fn get_by_uuid(
        &self,
        uuid: Uuid,
    ) -> BoxFuture<'_, Result<Option<UserCacheEntry>, StorageError>>;

    fn get_by_name<'a>(
        &'a self,
        name: &'a str,
    ) -> BoxFuture<'a, Result<Option<UserCacheEntry>, StorageError>>;
}
