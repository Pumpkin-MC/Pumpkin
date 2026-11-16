use serde::{Deserialize, Serialize};
/// Configuration for advancements
///
/// Controls whether the advancements should be saved and loaded
#[derive(Deserialize, Serialize, toml_spanner::Toml)]
#[serde(default)]
#[toml(ignore_unknown_fields, recoverable)]
pub struct AdvancementConfig {
    /// Whether advancements can be earned and shown to players.
    pub enabled: bool,
    /// Whether saving advancements is enabled.
    #[toml(default = Self::default().save_advancements)]
    pub save_advancements: bool,
}

impl Default for AdvancementConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            save_advancements: true,
        }
    }
}
