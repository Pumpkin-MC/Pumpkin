//! In-memory [`UserCacheStorage`] — `HashMap<Uuid, UserCacheEntry>`. Entries
//! expire 30 days after insert but are not proactively evicted.

use async_trait::async_trait;
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use crate::error::StorageError;
use crate::memory::MemoryStorage;
use crate::user_cache::{UserCacheEntry, UserCacheStorage};

fn one_month_from_now() -> OffsetDateTime {
    OffsetDateTime::now_utc() + Duration::days(30)
}

fn is_expired(entry: &UserCacheEntry) -> bool {
    OffsetDateTime::now_utc() >= entry.expiration_date
}

#[async_trait]
impl UserCacheStorage for MemoryStorage {
    async fn upsert(&self, uuid: Uuid, name: &str) -> Result<(), StorageError> {
        let entry = UserCacheEntry {
            uuid,
            name: name.to_string(),
            expiration_date: one_month_from_now(),
        };
        let mut guard = self.user_cache.write().await;
        guard.insert(uuid, entry);
        Ok(())
    }

    async fn get_by_uuid(&self, uuid: Uuid) -> Result<Option<UserCacheEntry>, StorageError> {
        let mut guard = self.user_cache.write().await;
        let Some(entry) = guard.get(&uuid) else {
            return Ok(None);
        };
        if is_expired(entry) {
            guard.remove(&uuid);
            return Ok(None);
        }
        Ok(Some(entry.clone()))
    }

    async fn get_by_name(&self, name: &str) -> Result<Option<UserCacheEntry>, StorageError> {
        let lower = name.to_ascii_lowercase();
        let mut guard = self.user_cache.write().await;
        let Some(hit_uuid) = guard
            .values()
            .find(|e| e.name.to_ascii_lowercase() == lower)
            .map(|e| e.uuid)
        else {
            return Ok(None);
        };
        let entry = guard.get(&hit_uuid).expect("just looked up");
        if is_expired(entry) {
            guard.remove(&hit_uuid);
            return Ok(None);
        }
        Ok(Some(entry.clone()))
    }
}
