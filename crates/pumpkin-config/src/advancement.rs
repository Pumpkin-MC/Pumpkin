use serde::{Deserialize, Serialize};
/// Configuration for advancements
///
/// Controls advancement tracking and persistence.
#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct AdvancementConfig {
    /// Whether advancements can be earned and shown to players.
    pub enabled: bool,
    /// Whether saving advancements is enabled.
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
