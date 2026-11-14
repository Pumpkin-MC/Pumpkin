//! Bukkit-style config file helpers for plugins.
//!
//! The host only exposes the plugin's private data folder
//! (`Context::get_data_folder`) plus file access through WASI. This module
//! adds the missing convention on top: `load_or_create_config` behaves like
//! `getConfig`/`saveDefaultConfig` — the first load writes the default file,
//! later loads read it back.
//!
//! Files are JSON and live directly under the data folder. Names are restricted
//! to `[a-zA-Z0-9._-]` with no directories so a plugin cannot escape its
//! folder through this API.

use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Serialize, de::DeserializeOwned};

/// Maximum config file size accepted on read.
const MAX_CONFIG_BYTES: u64 = 1024 * 1024;

/// Maximum config file name length.
const MAX_NAME_LEN: usize = 64;

static WRITE_NONCE: AtomicU64 = AtomicU64::new(0);

/// Checks that a config name is a plain file name.
fn validate_name(name: &str) -> Result<String, String> {
    if name.is_empty() || name.len() > MAX_NAME_LEN {
        return Err("config name must be 1-64 characters".to_string());
    }
    if name.contains('/') || name.contains('\\') || name.contains("..") {
        return Err("config name must not contain path separators".to_string());
    }
    let valid = name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-');
    if !valid {
        return Err("config name must match [a-zA-Z0-9._-]".to_string());
    }
    if name.starts_with('.') {
        return Err("config name must not start with '.'".to_string());
    }
    Ok(name.to_string())
}

/// Returns the full path for a config file inside the data folder.
fn config_path(data_folder: &str, name: &str) -> Result<PathBuf, String> {
    let file_name = validate_name(name)?;
    let mut path = PathBuf::from(data_folder);
    path.push(file_name);
    Ok(path)
}

/// Checks whether a config file exists.
pub fn config_exists(data_folder: &str, name: &str) -> bool {
    config_path(data_folder, name).is_ok_and(|path| path.is_file())
}

/// Loads a config file from the data folder.
pub fn load_config<T>(data_folder: &str, name: &str) -> Result<T, String>
where
    T: DeserializeOwned,
{
    let path = config_path(data_folder, name)?;
    let bytes = read_limited(&path)?;
    serde_json::from_slice(&bytes).map_err(|e| format!("parse config: {e}"))
}

/// Reads a file, rejecting it before parsing when it exceeds the size limit.
fn read_limited(path: &std::path::Path) -> Result<Vec<u8>, String> {
    let file = std::fs::File::open(path).map_err(|e| format!("read config: {e}"))?;
    let mut bytes = Vec::new();
    file.take(MAX_CONFIG_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| format!("read config: {e}"))?;
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > MAX_CONFIG_BYTES {
        return Err("config file too large".to_string());
    }
    Ok(bytes)
}

/// Saves a config file to the data folder, creating parent directories.
pub fn save_config<T>(data_folder: &str, name: &str, value: &T) -> Result<(), String>
where
    T: Serialize,
{
    let path = config_path(data_folder, name)?;
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent).map_err(|e| format!("create data folder: {e}"))?;
    }
    let bytes = serde_json::to_vec_pretty(value).map_err(|e| format!("encode config: {e}"))?;
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > MAX_CONFIG_BYTES {
        return Err("config file too large".to_string());
    }
    // Write to a temp file in the same directory, then rename, so a failed
    // write (e.g. full disk) never leaves a truncated config behind.
    let nonce = WRITE_NONCE.fetch_add(1, Ordering::Relaxed);
    let tmp_path = path.with_file_name(format!("{name}.tmp.{}-{nonce}", std::process::id()));
    if let Err(e) = std::fs::write(&tmp_path, &bytes) {
        let _ = std::fs::remove_file(&tmp_path);
        return Err(format!("write config: {e}"));
    }
    if let Err(e) = std::fs::rename(&tmp_path, &path) {
        let _ = std::fs::remove_file(&tmp_path);
        return Err(format!("write config: {e}"));
    }
    Ok(())
}

/// Loads a config file, writing `default` first when the file is missing.
///
/// This mirrors Bukkit's `saveDefaultConfig` + `getConfig` flow: call it in
/// `on_load` with the plugin's defaults, then work with the returned value.
///
/// Concurrent first-time callers race on exclusive creation: exactly one
/// writer wins and the losers load the winner's file.
pub fn load_or_create_config<T>(data_folder: &str, name: &str, default: &T) -> Result<T, String>
where
    T: Serialize + DeserializeOwned,
{
    let path = config_path(data_folder, name)?;
    let bytes = serde_json::to_vec_pretty(default).map_err(|e| format!("encode config: {e}"))?;
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > MAX_CONFIG_BYTES {
        return Err("config file too large".to_string());
    }
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent).map_err(|e| format!("create data folder: {e}"))?;
    }
    match std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
    {
        Ok(mut file) => {
            if let Err(e) = file.write_all(&bytes) {
                let _ = std::fs::remove_file(&path);
                return Err(format!("write config: {e}"));
            }
            serde_json::from_slice(&bytes).map_err(|e| format!("parse config: {e}"))
        }
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            reload_after_race(data_folder, name)
        }
        Err(e) => Err(format!("write config: {e}")),
    }
}

/// Reloads a config after losing a creation race, tolerating a winner whose
/// write has not finished yet.
fn reload_after_race<T>(data_folder: &str, name: &str) -> Result<T, String>
where
    T: DeserializeOwned,
{
    let mut last = load_config(data_folder, name);
    if last.is_ok() {
        return last;
    }
    for _ in 0..50 {
        std::thread::sleep(std::time::Duration::from_millis(1));
        last = load_config(data_folder, name);
        if last.is_ok() {
            return last;
        }
    }
    last
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_ID: AtomicU64 = AtomicU64::new(0);

    #[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize, Clone)]
    struct TestConfig {
        greeting: String,
        limit: u32,
    }

    fn unique_dir() -> PathBuf {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let dir =
            std::env::temp_dir().join(format!("pumpkin-config-test-{}-{id}", std::process::id()));
        if std::fs::create_dir_all(&dir).is_err() {
            return dir;
        }
        dir
    }

    fn cleanup(dir: &std::path::Path) {
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn save_and_load_round_trip() {
        let dir = unique_dir();
        let folder = dir.to_string_lossy().into_owned();
        let value = TestConfig {
            greeting: "hello".to_string(),
            limit: 42,
        };

        let result: Result<(), String> = save_config(&folder, "config.json", &value);
        assert!(result.is_ok());
        let loaded: Result<TestConfig, String> = load_config(&folder, "config.json");
        assert_eq!(loaded, Ok(value));

        cleanup(&dir);
    }

    #[test]
    fn load_or_create_writes_default_once() {
        let dir = unique_dir();
        let folder = dir.to_string_lossy().into_owned();
        let default = TestConfig {
            greeting: "hi".to_string(),
            limit: 7,
        };

        let first: TestConfig =
            load_or_create_config(&folder, "settings.json", &default).unwrap_or(default.clone());
        assert_eq!(first, default);
        assert!(config_exists(&folder, "settings.json"));

        let edited = TestConfig {
            greeting: "changed".to_string(),
            limit: 9,
        };
        assert!(save_config(&folder, "settings.json", &edited).is_ok());
        let second: Result<TestConfig, String> =
            load_or_create_config(&folder, "settings.json", &default);
        assert_eq!(second, Ok(edited));

        cleanup(&dir);
    }

    #[test]
    fn concurrent_load_or_create_agrees() {
        use std::sync::{Arc, Barrier};

        let dir = unique_dir();
        let folder = dir.to_string_lossy().into_owned();
        let barrier = Arc::new(Barrier::new(8));
        let handles: Vec<_> = (0u32..8)
            .map(|i| {
                let folder = folder.clone();
                let barrier = Arc::clone(&barrier);
                std::thread::spawn(move || {
                    barrier.wait();
                    let default = TestConfig {
                        greeting: format!("writer-{i}"),
                        limit: i,
                    };
                    load_or_create_config(&folder, "shared.json", &default)
                })
            })
            .collect();
        let results: Vec<TestConfig> = handles
            .into_iter()
            .map(|handle| {
                handle
                    .join()
                    .expect("thread panicked")
                    .expect("load failed")
            })
            .collect();
        assert!(results.windows(2).all(|pair| pair[0] == pair[1]));

        cleanup(&dir);
    }

    #[test]
    fn rejects_path_traversal_names() {
        let dir = unique_dir();
        let folder = dir.to_string_lossy().into_owned();
        let value = TestConfig {
            greeting: "x".to_string(),
            limit: 1,
        };

        for bad in ["../evil.json", "sub/config.json", ".hidden", "", "a/b"] {
            assert!(save_config(&folder, bad, &value).is_err(), "name: {bad}");
            assert!(!config_exists(&folder, bad));
        }

        cleanup(&dir);
    }

    #[test]
    fn load_missing_file_errors() {
        let dir = unique_dir();
        let folder = dir.to_string_lossy().into_owned();

        let loaded: Result<TestConfig, String> = load_config(&folder, "missing.json");
        assert!(loaded.is_err());

        cleanup(&dir);
    }

    #[test]
    fn rejects_oversized_file() {
        let dir = unique_dir();
        let folder = dir.to_string_lossy().into_owned();

        let big = vec![b'x'; 1024 * 1024 + 1];
        assert!(std::fs::write(dir.join("big.json"), big).is_ok());

        let loaded: Result<TestConfig, String> = load_config(&folder, "big.json");
        assert_eq!(loaded, Err("config file too large".to_string()));

        cleanup(&dir);
    }
}
