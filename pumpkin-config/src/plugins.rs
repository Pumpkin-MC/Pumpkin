use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default)]
#[serde(default)]
pub struct PluginsConfig {
    /// List of permissions that are globally blocked for all plugins.
    pub blocked_permissions: Vec<String>,
    /// List of plugins that are whitelisted.
    pub whitelist: Vec<WhitelistedPlugin>,
}

#[derive(Deserialize, Serialize)]
pub struct WhitelistedPlugin {
    pub name: String,
    pub version: String,
}
