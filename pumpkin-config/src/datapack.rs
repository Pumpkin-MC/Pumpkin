use serde::{Deserialize, Serialize};

/// Datapack-related configuration.
#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct DatapackConfig {
    /// Whether datapack detection, loading, and evaluation is enabled.
    pub enabled: bool,
    /// Whether to log detailed info when datapacks are loaded/reloaded
    /// (e.g. pack name, loot table count, function count, recipe count).
    pub log_load_info: bool,
}

impl Default for DatapackConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            log_load_info: true,
        }
    }
}
