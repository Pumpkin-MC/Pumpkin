use serde::{Deserialize, Serialize};

/// Configuration for player-versus-player mechanics.
///
/// Controls whether PVP is enabled, combat effects, and player protections.
#[derive(Deserialize, Serialize, toml_spanner::Toml)]
#[serde(default)]
#[toml(ignore_unknown_fields, recoverable)]
pub struct PVPConfig {
    /// Whether PVP is enabled on the server.
    #[toml(default = Self::default().enabled)]
    pub enabled: bool,
    /// Whether to show the red hurt animation and FOV bobbing when hit.
    #[toml(default = Self::default().hurt_animation)]
    pub hurt_animation: bool,
    /// Whether players in creative mode are protected from PVP.
    #[toml(default = Self::default().protect_creative)]
    pub protect_creative: bool,
    /// Whether knockback from attacks is applied.
    #[toml(default = Self::default().knockback)]
    pub knockback: bool,
    /// Whether players swing their hand when attacking.
    #[toml(default = Self::default().swing)]
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
