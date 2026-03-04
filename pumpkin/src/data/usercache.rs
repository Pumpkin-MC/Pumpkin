use std::cmp::Reverse;
use std::collections::{HashMap, HashSet};
use std::{env, fs};

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use tracing::{debug, warn};
use uuid::Uuid;

const USER_CACHE_PATH: &str = "usercache.json";
const USER_CACHE_TTL_SECONDS: i64 = 60 * 60 * 24 * 30;
const USER_CACHE_LIMIT: usize = 1000;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserCacheEntry {
    pub uuid: Uuid,
    pub name: String,
    pub expires_on: i64,
    pub last_access: i64,
}

impl UserCacheEntry {
    #[allow(clippy::missing_const_for_fn)]
    fn new(uuid: Uuid, name: String, now: i64) -> Self {
        Self {
            uuid,
            name,
            expires_on: now + USER_CACHE_TTL_SECONDS,
            last_access: now,
        }
    }

    const fn is_expired(&self, now: i64) -> bool {
        self.expires_on < now
    }
}

#[derive(Default, Deserialize, Serialize)]
#[serde(transparent)]
pub struct UserCache {
    entries: Vec<UserCacheEntry>,
}

impl UserCache {
    fn path() -> std::path::PathBuf {
        env::current_dir()
            .unwrap_or_else(|_| ".".into())
            .join(super::DATA_FOLDER)
            .join(USER_CACHE_PATH)
    }

    #[must_use]
    pub fn load() -> Self {
        let path = Self::path();
        let data_dir = path
            .parent()
            .expect("usercache path should always have a parent directory");
        if !data_dir.exists() && let Err(error) = fs::create_dir_all(data_dir) {
            warn!("Failed to create data directory {}: {error}", data_dir.display());
            return Self::default();
        }

        let mut cache = if path.exists() {
            fs::read_to_string(&path)
                .ok()
                .and_then(|content| serde_json::from_str::<Self>(&content).ok())
                .unwrap_or_else(|| {
                    warn!(
                        "Failed to parse user cache at {}, resetting file",
                        path.display()
                    );
                    Self::default()
                })
        } else {
            Self::default()
        };

        cache.normalize();
        cache.save();
        cache
    }

    pub fn save(&self) {
        let path = Self::path();
        if let Some(parent) = path.parent()
            && !parent.exists()
            && let Err(error) = fs::create_dir_all(parent)
        {
            warn!(
                "Failed to create usercache parent directory {}: {error}",
                parent.display()
            );
            return;
        }

        let Ok(content) = serde_json::to_string_pretty(self) else {
            warn!("Failed to serialize user cache");
            return;
        };

        if let Err(error) = fs::write(&path, content) {
            warn!("Failed to save user cache to {}: {error}", path.display());
        }
    }

    pub fn upsert(&mut self, uuid: Uuid, name: String) {
        let now = OffsetDateTime::now_utc().unix_timestamp();
        if let Some(existing) = self.entries.iter_mut().find(|entry| entry.uuid == uuid) {
            existing.name = name;
            existing.last_access = now;
            existing.expires_on = now + USER_CACHE_TTL_SECONDS;
        } else {
            self.entries.push(UserCacheEntry::new(uuid, name, now));
        }

        self.normalize();
        self.save();
    }

    pub fn get_by_name(&mut self, name: &str) -> Option<UserCacheEntry> {
        let now = OffsetDateTime::now_utc().unix_timestamp();
        self.normalize();

        let idx = self
            .entries
            .iter()
            .position(|entry| entry.name.eq_ignore_ascii_case(name))?;

        let entry = self.entries.get_mut(idx)?;
        entry.last_access = now;
        entry.expires_on = now + USER_CACHE_TTL_SECONDS;

        let result = entry.clone();
        self.normalize();
        self.save();
        Some(result)
    }

    pub fn get_by_uuid(&mut self, uuid: Uuid) -> Option<UserCacheEntry> {
        self.normalize();
        let now = OffsetDateTime::now_utc().unix_timestamp();

        let idx = self.entries.iter().position(|entry| entry.uuid == uuid)?;
        let entry = self.entries.get_mut(idx)?;
        entry.last_access = now;
        entry.expires_on = now + USER_CACHE_TTL_SECONDS;

        let result = entry.clone();
        self.normalize();
        self.save();
        Some(result)
    }

    pub fn names(&mut self) -> Vec<String> {
        self.normalize();
        self.entries.iter().map(|entry| entry.name.clone()).collect()
    }

    fn normalize(&mut self) {
        let now = OffsetDateTime::now_utc().unix_timestamp();
        self.entries.retain(|entry| !entry.is_expired(now));

        // Keep most recently accessed entries first and deduplicate by UUID/name.
        self.entries.sort_by_key(|entry| Reverse(entry.last_access));
        let mut seen_uuid = HashSet::new();
        let mut seen_name = HashSet::<String>::new();
        self.entries.retain(|entry| {
            let lowercase = entry.name.to_ascii_lowercase();
            seen_uuid.insert(entry.uuid) && seen_name.insert(lowercase)
        });

        if self.entries.len() > USER_CACHE_LIMIT {
            debug!(
                "Truncating usercache from {} to {} entries",
                self.entries.len(),
                USER_CACHE_LIMIT
            );
            self.entries.truncate(USER_CACHE_LIMIT);
        }

        // Preserve deterministic ordering for file diffs after truncation.
        self.entries.sort_by_key(|entry| Reverse(entry.last_access));
    }

    #[must_use]
    pub fn lookup_tables(&self) -> (HashMap<String, Uuid>, HashMap<Uuid, UserCacheEntry>) {
        let mut name_to_uuid = HashMap::with_capacity(self.entries.len());
        let mut uuid_to_entry = HashMap::with_capacity(self.entries.len());

        for entry in &self.entries {
            name_to_uuid.insert(entry.name.to_ascii_lowercase(), entry.uuid);
            uuid_to_entry.insert(entry.uuid, entry.clone());
        }

        (name_to_uuid, uuid_to_entry)
    }
}
