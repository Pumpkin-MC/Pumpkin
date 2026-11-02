use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PermissionCacheEntry {
    pub permissions_requested: Vec<String>,
    pub approved: bool,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct PermissionCache {
    pub entries: HashMap<String, PermissionCacheEntry>, // Key is the hex hash of the plugin
}

impl PermissionCache {
    pub async fn load(path: &Path) -> Self {
        fs::read_to_string(path).await.map_or_else(
            |_| Self::default(),
            |data| serde_json::from_str(&data).unwrap_or_default(),
        )
    }

    pub async fn save(&self, path: &Path) -> tokio::io::Result<()> {
        let data = serde_json::to_string_pretty(self).map_err(std::io::Error::other)?;
        fs::write(path, data).await
    }
}

pub async fn calculate_hash(path: &Path) -> tokio::io::Result<String> {
    let bytes = fs::read(path).await?;
    Ok(calculate_hash_for_bytes(&bytes))
}

#[must_use]
pub fn calculate_hash_for_bytes(bytes: &[u8]) -> String {
    use std::fmt::Write;

    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let result = hasher.finalize();
    result.iter().fold(String::new(), |mut output, b| {
        let _ = write!(output, "{b:02x}");
        output
    })
}
