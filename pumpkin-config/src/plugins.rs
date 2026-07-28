use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default, Clone)]
#[serde(default)]
pub struct PluginsConfig {
    /// List of permissions that are globally blocked for all plugins.
    pub blocked_permissions: Vec<String>,
}
