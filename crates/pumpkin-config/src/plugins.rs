use serde::{Deserialize, Serialize};

/// Plugin system configuration.
#[derive(Deserialize, Serialize, Default)]
#[serde(default)]
pub struct PluginsConfig {
    /// List of permissions that are globally blocked for all plugins.
    pub blocked_permissions: Vec<String>,
}
