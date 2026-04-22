//! Vanilla-compatible `usercache.json` MRU cache with a 1000-entry soft cap.
//! Loaded lazily on first access; snapshots keep the most-recently-used
//! entries.

use std::cmp::Reverse;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use time::format_description::well_known::Rfc3339;
use time::{Duration, OffsetDateTime};
use tokio::fs;
use uuid::Uuid;

use crate::error::StorageError;
use crate::user_cache::{UserCacheEntry, UserCacheStorage};
use crate::vanilla::VanillaStorage;

const USER_CACHE_FILE: &str = "usercache.json";
const USER_CACHE_MRU_LIMIT: usize = 1000;

#[derive(Debug, Default)]
pub(crate) struct UserCacheInner {
    loaded: bool,
    profiles_by_name: HashMap<String, InternalEntry>,
    profiles_by_uuid: HashMap<Uuid, InternalEntry>,
    operation_count: u64,
}

#[derive(Debug, Clone)]
struct InternalEntry {
    uuid: Uuid,
    name: String,
    expiration_date: OffsetDateTime,
    last_access: u64,
}

impl InternalEntry {
    fn to_public(&self) -> UserCacheEntry {
        UserCacheEntry {
            uuid: self.uuid,
            name: self.name.clone(),
            expiration_date: self.expiration_date,
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct UserCacheEntryDisk {
    uuid: Uuid,
    name: String,
    expires_on: String,
}

impl VanillaStorage {
    fn user_cache_path(&self) -> PathBuf {
        self.server_data_dir().join(USER_CACHE_FILE)
    }
}

impl UserCacheInner {
    async fn ensure_loaded(&mut self, path: &Path) -> Result<(), StorageError> {
        if self.loaded {
            return Ok(());
        }
        let mut entries = load_entries(path).await?;
        entries.reverse();
        for e in entries {
            self.safe_add(e);
        }
        self.loaded = true;
        Ok(())
    }

    fn next_operation(&mut self) -> u64 {
        self.operation_count += 1;
        self.operation_count
    }

    fn safe_add(&mut self, mut entry: InternalEntry) {
        entry.last_access = self.next_operation();
        self.profiles_by_name
            .insert(entry.name.to_ascii_lowercase(), entry.clone());
        self.profiles_by_uuid.insert(entry.uuid, entry);
    }

    fn add_fresh(&mut self, uuid: Uuid, name: String) -> InternalEntry {
        let entry = InternalEntry {
            uuid,
            name,
            expiration_date: one_month_from_now(),
            last_access: 0,
        };
        self.safe_add(entry.clone());
        entry
    }

    fn top_mru_profiles(&self, limit: usize) -> Vec<InternalEntry> {
        let mut entries: Vec<InternalEntry> = self.profiles_by_uuid.values().cloned().collect();
        entries.sort_by_key(|entry| Reverse(entry.last_access));
        entries.truncate(limit);
        entries
    }
}

async fn load_entries(path: &Path) -> Result<Vec<InternalEntry>, StorageError> {
    let raw = match fs::read_to_string(path).await {
        Ok(raw) => raw,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(StorageError::io_at(path, e)),
    };

    let Ok(json) = serde_json::from_str::<serde_json::Value>(&raw) else {
        return Ok(Vec::new());
    };
    let Some(array) = json.as_array() else {
        return Ok(Vec::new());
    };

    let mut entries = Vec::new();
    for element in array {
        let Some(object) = element.as_object() else {
            continue;
        };
        let Some(name) = object.get("name").and_then(serde_json::Value::as_str) else {
            continue;
        };
        let Some(uuid_raw) = object.get("uuid").and_then(serde_json::Value::as_str) else {
            continue;
        };
        let Some(expires_on) = object.get("expiresOn").and_then(serde_json::Value::as_str) else {
            continue;
        };
        let Ok(uuid) = Uuid::parse_str(uuid_raw) else {
            continue;
        };
        let Ok(expiration_date) = OffsetDateTime::parse(expires_on, &Rfc3339) else {
            continue;
        };
        entries.push(InternalEntry {
            uuid,
            name: name.to_string(),
            expiration_date,
            last_access: 0,
        });
    }
    Ok(entries)
}

async fn save_snapshot(path: &Path, snapshot: Vec<InternalEntry>) -> Result<(), StorageError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .await
            .map_err(|e| StorageError::io_at(parent, e))?;
    }
    let to_save: Vec<UserCacheEntryDisk> = snapshot
        .into_iter()
        .map(|entry| UserCacheEntryDisk {
            uuid: entry.uuid,
            name: entry.name,
            expires_on: entry
                .expiration_date
                .format(&Rfc3339)
                .expect("Rfc3339 format is infallible for OffsetDateTime"),
        })
        .collect();
    let content = serde_json::to_string(&to_save)
        .map_err(|e| StorageError::Serialize(e.to_string()))?;
    fs::write(path, content)
        .await
        .map_err(|e| StorageError::io_at(path, e))?;
    Ok(())
}

fn is_expired(expiration_date: OffsetDateTime) -> bool {
    OffsetDateTime::now_utc() >= expiration_date
}

fn one_month_from_now() -> OffsetDateTime {
    OffsetDateTime::now_utc() + Duration::days(30)
}

#[async_trait]
impl UserCacheStorage for VanillaStorage {
    async fn upsert(&self, uuid: Uuid, name: &str) -> Result<(), StorageError> {
        let path = self.user_cache_path();
        let mut guard = self.user_cache_inner.lock().await;
        guard.ensure_loaded(&path).await?;
        guard.add_fresh(uuid, name.to_string());
        let snapshot = guard.top_mru_profiles(USER_CACHE_MRU_LIMIT);
        drop(guard);
        save_snapshot(&path, snapshot).await
    }

    async fn get_by_uuid(&self, uuid: Uuid) -> Result<Option<UserCacheEntry>, StorageError> {
        let path = self.user_cache_path();
        let mut guard = self.user_cache_inner.lock().await;
        guard.ensure_loaded(&path).await?;

        let Some(mut entry) = guard.profiles_by_uuid.get(&uuid).cloned() else {
            return Ok(None);
        };
        entry.last_access = guard.next_operation();
        guard
            .profiles_by_name
            .insert(entry.name.to_ascii_lowercase(), entry.clone());
        guard.profiles_by_uuid.insert(entry.uuid, entry.clone());
        Ok(Some(entry.to_public()))
    }

    async fn get_by_name(&self, name: &str) -> Result<Option<UserCacheEntry>, StorageError> {
        let path = self.user_cache_path();
        let mut guard = self.user_cache_inner.lock().await;
        guard.ensure_loaded(&path).await?;

        let lowercase_name = name.to_ascii_lowercase();
        let lookup = guard.profiles_by_name.get(&lowercase_name).cloned();

        let (profile, needs_save) = if let Some(entry) = lookup {
            if is_expired(entry.expiration_date) {
                guard.profiles_by_uuid.remove(&entry.uuid);
                guard
                    .profiles_by_name
                    .remove(&entry.name.to_ascii_lowercase());
                (None, true)
            } else {
                (Some(entry), false)
            }
        } else {
            (None, false)
        };

        let Some(mut entry) = profile else {
            if needs_save {
                let snapshot = guard.top_mru_profiles(USER_CACHE_MRU_LIMIT);
                drop(guard);
                save_snapshot(&path, snapshot).await?;
            }
            return Ok(None);
        };

        entry.last_access = guard.next_operation();
        guard
            .profiles_by_name
            .insert(entry.name.to_ascii_lowercase(), entry.clone());
        guard.profiles_by_uuid.insert(entry.uuid, entry.clone());
        Ok(Some(entry.to_public()))
    }
}
