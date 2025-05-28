use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct PVPConfig {
    /// Whether PVP is enabled.
    pub enabled: bool,
    /// Whether to use the red hurt animation and FOV bobbing.
    pub hurt_animation: bool,
    /// Whether players in creative mode are protected against PVP.
    pub protect_creative: bool,
    /// Whether PVP knockback is enabled.
    pub knockback: bool,
    /// Whether players swing when attacking.
    pub swing: bool,
}

impl Default for PVPConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            hurt_animation: true,
            protect_creative: true,
            knockback: true,
            swing: true,
        }
    }
}
