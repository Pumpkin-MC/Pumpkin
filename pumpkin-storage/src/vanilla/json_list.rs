//! Shared pretty-printed JSON list helpers used by the banlist/op/whitelist
//! vanilla backends. All four domains persist a flat `Vec<T>` to a single
//! file and share the "missing file == empty list" convention.

use std::path::Path;

use serde::{Serialize, de::DeserializeOwned};
use tokio::fs;

use crate::error::StorageError;

/// Reads a `Vec<T>` from `path`. A missing file yields an empty list to match
/// vanilla's "absent == empty" semantics.
pub(super) async fn load_json_list<T: DeserializeOwned>(
    path: &Path,
) -> Result<Vec<T>, StorageError> {
    match fs::read_to_string(path).await {
        Ok(content) => serde_json::from_str::<Vec<T>>(&content)
            .map_err(|e| StorageError::Deserialize(e.to_string())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(e) => Err(StorageError::io_at(path, e)),
    }
}

/// Writes `entries` to `path` as pretty-printed JSON, creating the parent
/// directory if necessary.
pub(super) async fn save_json_list<T: Serialize>(
    path: &Path,
    entries: &[T],
) -> Result<(), StorageError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .await
            .map_err(|e| StorageError::io_at(parent, e))?;
    }
    let content = serde_json::to_string_pretty(entries)
        .map_err(|e| StorageError::Serialize(e.to_string()))?;
    fs::write(path, content)
        .await
        .map_err(|e| StorageError::io_at(path, e))?;
    Ok(())
}
