use std::cmp::Reverse;
use std::collections::HashMap;
use std::{env, fs};

use chrono::{DateTime, FixedOffset, Local, Months, TimeDelta, Utc};
use serde::{Deserialize, Serialize};
use tracing::warn;
use uuid::Uuid;

const USER_CACHE_PATH: &str = "usercache.json";
const USER_CACHE_MRU_LIMIT: usize = 1000;
const USER_CACHE_DATE_FORMAT: &str = "%Y-%m-%d %H:%M:%S %z";

#[derive(Clone, Debug)]
pub struct UserCacheEntry {
    pub uuid: Uuid,
    pub name: String,
    expiration_date: DateTime<FixedOffset>,
    last_access: u64,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct UserCacheEntryDisk {
    uuid: Uuid,
    name: String,
    expires_on: String,
}

#[derive(Default)]
pub struct UserCache {
    profiles_by_name: HashMap<String, UserCacheEntry>,
    profiles_by_uuid: HashMap<Uuid, UserCacheEntry>,
    operation_count: u64,
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
        let mut cache = Self::default();
        let mut loaded = Self::load_entries();
        loaded.reverse();
        for entry in loaded {
            cache.safe_add(entry);
        }
        cache
    }

    pub fn save(&self) {
        let path = Self::path();
        if let Some(parent) = path.parent()
            && let Err(error) = fs::create_dir_all(parent)
        {
            warn!("Failed to create user cache directory: {error}");
            return;
        }

        let to_save: Vec<UserCacheEntryDisk> = self
            .top_mru_profiles(USER_CACHE_MRU_LIMIT)
            .into_iter()
            .map(|entry| UserCacheEntryDisk {
                uuid: entry.uuid,
                name: entry.name,
                expires_on: format_cache_date(entry.expiration_date),
            })
            .collect();

        let Ok(content) = serde_json::to_string(&to_save) else {
            return;
        };

        if let Err(error) = fs::write(path, content) {
            warn!("Failed to save user cache: {error}");
        }
    }

    pub fn upsert(&mut self, uuid: Uuid, name: String) {
        self.add_internal(uuid, name);
    }

    pub fn get_by_name(&mut self, name: &str) -> Option<UserCacheEntry> {
        let lowercase_name = name.to_ascii_lowercase();
        let mut profile = self.profiles_by_name.get(&lowercase_name).cloned();
        let mut needs_save = false;

        if let Some(entry) = &profile
            && is_expired(entry.expiration_date)
        {
            self.profiles_by_uuid.remove(&entry.uuid);
            self.profiles_by_name
                .remove(&entry.name.to_ascii_lowercase());
            needs_save = true;
            profile = None;
        }

        if let Some(mut entry) = profile {
            entry.last_access = self.next_operation();
            self.profiles_by_name
                .insert(entry.name.to_ascii_lowercase(), entry.clone());
            self.profiles_by_uuid.insert(entry.uuid, entry.clone());
            return Some(entry);
        }

        if needs_save {
            self.save();
        }

        None
    }

    pub fn get_by_uuid(&mut self, uuid: Uuid) -> Option<UserCacheEntry> {
        let mut entry = self.profiles_by_uuid.get(&uuid).cloned()?;
        entry.last_access = self.next_operation();
        self.profiles_by_name
            .insert(entry.name.to_ascii_lowercase(), entry.clone());
        self.profiles_by_uuid.insert(entry.uuid, entry.clone());
        Some(entry)
    }

    fn add_internal(&mut self, uuid: Uuid, name: String) -> UserCacheEntry {
        let expiration_date = one_month_from_now();
        let entry = UserCacheEntry {
            uuid,
            name,
            expiration_date,
            last_access: 0,
        };

        self.safe_add(entry.clone());
        self.save();
        entry
    }

    fn safe_add(&mut self, mut entry: UserCacheEntry) {
        entry.last_access = self.next_operation();
        self.profiles_by_name
            .insert(entry.name.to_ascii_lowercase(), entry.clone());
        self.profiles_by_uuid.insert(entry.uuid, entry);
    }

    #[allow(clippy::missing_const_for_fn)]
    fn next_operation(&mut self) -> u64 {
        self.operation_count += 1;
        self.operation_count
    }

    fn top_mru_profiles(&self, limit: usize) -> Vec<UserCacheEntry> {
        let mut entries: Vec<UserCacheEntry> = self.profiles_by_uuid.values().cloned().collect();
        entries.sort_by_key(|entry| Reverse(entry.last_access));
        entries.truncate(limit);
        entries
    }

    fn load_entries() -> Vec<UserCacheEntry> {
        let path = Self::path();
        let Ok(raw) = fs::read_to_string(path) else {
            return Vec::new();
        };

        let Ok(json) = serde_json::from_str::<serde_json::Value>(&raw) else {
            return Vec::new();
        };

        let Some(array) = json.as_array() else {
            return Vec::new();
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
            let Some(expires_on) = object.get("expiresOn").and_then(serde_json::Value::as_str)
            else {
                continue;
            };

            let Ok(uuid) = Uuid::parse_str(uuid_raw) else {
                continue;
            };
            let Ok(expiration_date) = DateTime::parse_from_str(expires_on, USER_CACHE_DATE_FORMAT)
            else {
                continue;
            };

            entries.push(UserCacheEntry {
                uuid,
                name: name.to_string(),
                expiration_date,
                last_access: 0,
            });
        }

        entries
    }
}

fn format_cache_date(date: DateTime<FixedOffset>) -> String {
    date.format(USER_CACHE_DATE_FORMAT).to_string()
}

fn is_expired(expiration_date: DateTime<FixedOffset>) -> bool {
    Utc::now() >= expiration_date.with_timezone(&Utc)
}

fn one_month_from_now() -> DateTime<FixedOffset> {
    let now = Local::now().fixed_offset();
    now.checked_add_months(Months::new(1))
        .unwrap_or(now + TimeDelta::days(30))
}
