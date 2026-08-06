use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::plugin::loader::wasm::wasm_host::{
    state::PluginHostState, wit::v0_1::pumpkin::plugin::storage::Host,
};
use crate::plugin::permissions::{FS_READ_DATA, FS_WRITE_DATA};

/// The file name used for a plugin's key-value store inside its data folder.
const STORAGE_FILE: &str = "storage.json";

/// Checks whether the given plugin permissions allow a storage operation.
///
/// The store lives in the plugin's data folder, so it is governed by the same
/// permissions as direct file access: reading needs `fs.read.data` (writing
/// also grants read-back), and writing needs `fs.write.data`. This keeps the
/// convenience store from being a way around the file-system permission gate.
fn check_storage_permission(permissions: &[String], write: bool) -> Result<(), String> {
    let has = |permission: &str| permissions.iter().any(|held| held == permission);

    if write {
        if has(FS_WRITE_DATA) {
            Ok(())
        } else {
            Err(format!(
                "writing to plugin storage requires the \"{FS_WRITE_DATA}\" permission"
            ))
        }
    } else if has(FS_READ_DATA) || has(FS_WRITE_DATA) {
        Ok(())
    } else {
        Err(format!(
            "reading plugin storage requires the \"{FS_READ_DATA}\" permission"
        ))
    }
}

/// Turns a plugin name into a safe single path segment.
///
/// Plugin names come from plugin metadata and are used to build a folder path,
/// so anything that could escape the plugins directory (path separators, `..`,
/// and similar) is replaced. The result is always a non-empty, plain segment.
fn sanitize_plugin_name(name: &str) -> String {
    let mut sanitized: String = name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.') {
                c
            } else {
                '_'
            }
        })
        .collect();

    // Guard against names that are empty or that resolve to the current or
    // parent directory once the disallowed characters are replaced.
    if sanitized.is_empty() || sanitized.chars().all(|c| c == '.') {
        sanitized = "plugin".to_string();
    }

    sanitized
}

/// Returns the path to a plugin's key-value store file.
fn storage_path_for(plugin_name: &str) -> PathBuf {
    Path::new("plugins")
        .join("data")
        .join(sanitize_plugin_name(plugin_name))
        .join(STORAGE_FILE)
}

/// A plugin's on-disk key-value store, backed by a JSON file.
///
/// The whole store is kept in memory and written back to disk after every
/// change, so data survives a restart. Writes go to a temporary file that is
/// then renamed over the real one, so a crash mid-write cannot leave a
/// half-written file behind.
pub struct PluginKvStore {
    file_path: PathBuf,
    data: HashMap<String, String>,
}

impl PluginKvStore {
    /// Loads the store from `file_path`, starting empty if the file does not
    /// exist yet.
    pub async fn load(file_path: PathBuf) -> std::io::Result<Self> {
        let data = match tokio::fs::read(&file_path).await {
            Ok(bytes) => serde_json::from_slice(&bytes).map_err(std::io::Error::other)?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => HashMap::new(),
            Err(error) => return Err(error),
        };

        Ok(Self { file_path, data })
    }

    /// Returns the value stored under `key`, if any.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&String> {
        self.data.get(key)
    }

    /// Returns `true` if `key` currently has a value.
    #[must_use]
    pub fn contains(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }

    /// Returns every stored key, optionally limited to keys starting with
    /// `prefix`.
    #[must_use]
    pub fn keys(&self, prefix: Option<&str>) -> Vec<String> {
        self.data
            .keys()
            .filter(|key| prefix.is_none_or(|prefix| key.starts_with(prefix)))
            .cloned()
            .collect()
    }

    /// Stores `value` under `key`, replacing any existing value, and persists
    /// the change.
    pub async fn set(&mut self, key: String, value: String) -> std::io::Result<()> {
        self.data.insert(key, value);
        self.persist().await
    }

    /// Removes `key`, persisting the change. Returns `true` if the key existed.
    pub async fn remove(&mut self, key: &str) -> std::io::Result<bool> {
        if self.data.remove(key).is_some() {
            self.persist().await?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Removes every key, persisting the change.
    pub async fn clear(&mut self) -> std::io::Result<()> {
        if self.data.is_empty() {
            return Ok(());
        }
        self.data.clear();
        self.persist().await
    }

    /// Writes the store to disk atomically (temporary file, then rename).
    async fn persist(&self) -> std::io::Result<()> {
        if let Some(parent) = self.file_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let json = serde_json::to_vec_pretty(&self.data).map_err(std::io::Error::other)?;

        let mut temp_path = self.file_path.clone();
        let mut file_name = temp_path.file_name().unwrap_or_default().to_os_string();
        file_name.push(".tmp");
        temp_path.set_file_name(file_name);

        tokio::fs::write(&temp_path, &json).await?;
        tokio::fs::rename(&temp_path, &self.file_path).await
    }
}

impl PluginHostState {
    /// Lazily loads and returns this plugin's key-value store.
    ///
    /// The store is only read from disk the first time a plugin touches it, so
    /// plugins that never use storage pay nothing for it.
    async fn kv_store(&mut self) -> Result<&mut PluginKvStore, String> {
        if self.storage.is_none() {
            let plugin_name = self
                .plugin_name
                .as_deref()
                .ok_or_else(|| "plugin storage is not available yet".to_string())?;
            let store = PluginKvStore::load(storage_path_for(plugin_name))
                .await
                .map_err(|error| format!("failed to open plugin storage: {error}"))?;
            self.storage = Some(store);
        }

        Ok(self
            .storage
            .as_mut()
            .expect("storage was just initialized above"))
    }
}

impl Host for PluginHostState {
    async fn set(&mut self, key: String, value: String) -> wasmtime::Result<Result<(), String>> {
        if let Err(error) = check_storage_permission(&self.permissions, true) {
            return Ok(Err(error));
        }
        let store = match self.kv_store().await {
            Ok(store) => store,
            Err(error) => return Ok(Err(error)),
        };
        Ok(store
            .set(key, value)
            .await
            .map_err(|error| format!("failed to write plugin storage: {error}")))
    }

    async fn get(&mut self, key: String) -> wasmtime::Result<Result<Option<String>, String>> {
        if let Err(error) = check_storage_permission(&self.permissions, false) {
            return Ok(Err(error));
        }
        let store = match self.kv_store().await {
            Ok(store) => store,
            Err(error) => return Ok(Err(error)),
        };
        Ok(Ok(store.get(&key).cloned()))
    }

    async fn remove(&mut self, key: String) -> wasmtime::Result<Result<bool, String>> {
        if let Err(error) = check_storage_permission(&self.permissions, true) {
            return Ok(Err(error));
        }
        let store = match self.kv_store().await {
            Ok(store) => store,
            Err(error) => return Ok(Err(error)),
        };
        Ok(store
            .remove(&key)
            .await
            .map_err(|error| format!("failed to write plugin storage: {error}")))
    }

    async fn contains(&mut self, key: String) -> wasmtime::Result<Result<bool, String>> {
        if let Err(error) = check_storage_permission(&self.permissions, false) {
            return Ok(Err(error));
        }
        let store = match self.kv_store().await {
            Ok(store) => store,
            Err(error) => return Ok(Err(error)),
        };
        Ok(Ok(store.contains(&key)))
    }

    async fn keys(
        &mut self,
        prefix: Option<String>,
    ) -> wasmtime::Result<Result<Vec<String>, String>> {
        if let Err(error) = check_storage_permission(&self.permissions, false) {
            return Ok(Err(error));
        }
        let store = match self.kv_store().await {
            Ok(store) => store,
            Err(error) => return Ok(Err(error)),
        };
        Ok(Ok(store.keys(prefix.as_deref())))
    }

    async fn clear(&mut self) -> wasmtime::Result<Result<(), String>> {
        if let Err(error) = check_storage_permission(&self.permissions, true) {
            return Ok(Err(error));
        }
        let store = match self.kv_store().await {
            Ok(store) => store,
            Err(error) => return Ok(Err(error)),
        };
        Ok(store
            .clear()
            .await
            .map_err(|error| format!("failed to write plugin storage: {error}")))
    }
}

#[cfg(test)]
mod test {
    use super::{
        FS_READ_DATA, FS_WRITE_DATA, PluginKvStore, check_storage_permission, sanitize_plugin_name,
        storage_path_for,
    };

    #[test]
    fn storage_permissions_are_enforced() {
        let none: Vec<String> = Vec::new();
        let read_only = vec![FS_READ_DATA.to_string()];
        let write = vec![FS_WRITE_DATA.to_string()];

        // No file-system permission means neither reads nor writes are allowed.
        assert!(check_storage_permission(&none, false).is_err());
        assert!(check_storage_permission(&none, true).is_err());

        // Read permission allows reads but not writes.
        assert!(check_storage_permission(&read_only, false).is_ok());
        assert!(check_storage_permission(&read_only, true).is_err());

        // Write permission allows both writes and read-back.
        assert!(check_storage_permission(&write, true).is_ok());
        assert!(check_storage_permission(&write, false).is_ok());
    }

    #[test]
    fn sanitize_rejects_path_traversal() {
        // Path separators are always replaced, so a sanitized name can never be
        // more than a single path segment (the key safety property).
        assert!(!sanitize_plugin_name("../../etc").contains(['/', '\\']));
        assert_eq!(sanitize_plugin_name("my/evil\\name"), "my_evil_name");
        // Names that are nothing but dots (`.`, `..`) or empty would resolve to
        // the current or parent directory, so they fall back to a safe default.
        assert_eq!(sanitize_plugin_name(".."), "plugin");
        assert_eq!(sanitize_plugin_name("."), "plugin");
        assert_eq!(sanitize_plugin_name(""), "plugin");
        // Ordinary names are left intact.
        assert_eq!(
            sanitize_plugin_name("Better-Trial_Chambers.1"),
            "Better-Trial_Chambers.1"
        );
    }

    #[test]
    fn storage_path_stays_inside_the_data_folder() {
        let path = storage_path_for("../escape");
        // The traversal collapses to a single harmless segment under plugins/data.
        assert!(path.ends_with("plugins/data/.._escape/storage.json"));
    }

    #[tokio::test]
    async fn set_get_remove_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("storage.json");

        let mut store = PluginKvStore::load(path.clone()).await.unwrap();
        assert_eq!(store.get("a"), None);

        store.set("a".to_string(), "1".to_string()).await.unwrap();
        store.set("b".to_string(), "2".to_string()).await.unwrap();
        assert_eq!(store.get("a"), Some(&"1".to_string()));
        assert!(store.contains("b"));

        assert!(store.remove("a").await.unwrap());
        assert!(!store.remove("a").await.unwrap());
        assert_eq!(store.get("a"), None);
    }

    #[tokio::test]
    async fn values_persist_across_reload() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("storage.json");

        let mut store = PluginKvStore::load(path.clone()).await.unwrap();
        store
            .set("player:uuid".to_string(), "{\"vaults\":3}".to_string())
            .await
            .unwrap();
        drop(store);

        let reloaded = PluginKvStore::load(path).await.unwrap();
        assert_eq!(reloaded.get("player:uuid"), Some(&"{\"vaults\":3}".to_string()));
    }

    #[tokio::test]
    async fn keys_can_be_filtered_by_prefix() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("storage.json");

        let mut store = PluginKvStore::load(path).await.unwrap();
        store.set("player:a".to_string(), "1".to_string()).await.unwrap();
        store.set("player:b".to_string(), "2".to_string()).await.unwrap();
        store.set("chamber:1".to_string(), "x".to_string()).await.unwrap();

        let mut players = store.keys(Some("player:"));
        players.sort();
        assert_eq!(players, vec!["player:a".to_string(), "player:b".to_string()]);
        assert_eq!(store.keys(None).len(), 3);

        store.clear().await.unwrap();
        assert!(store.keys(None).is_empty());
    }
}
