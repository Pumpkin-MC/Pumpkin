use serde::{Deserialize, Serialize};

/// Configuration for player-versus-player mechanics.
///
/// Controls whether PVP is enabled, combat effects, and player protections.
#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct PVPConfig {
    /// Whether PVP is enabled on the server.
    pub enabled: bool,
    /// Whether to show the red hurt animation and FOV bobbing when hit.
    pub hurt_animation: bool,
    /// Whether players in creative mode are protected from PVP.
    pub protect_creative: bool,
    /// Whether knockback from attacks is applied.
    pub knockback: bool,
    /// Whether players swing their hand when attacking.
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
