//! Public entry type for the user cache (`usercache.json`).
//!
//! MRU bookkeeping and on-disk storage is an implementation detail of the
//! future `UserCacheStorage` impl; callers only see the fields defined here.

use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct UserCacheEntry {
    pub uuid: Uuid,
    pub name: String,
    pub expiration_date: OffsetDateTime,
}
