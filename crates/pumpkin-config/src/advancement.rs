use serde::{Deserialize, Serialize};
/// Configuration for advancements
///
/// Controls whether the advancements should be saved and loaded
#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct AdvancementConfig {
    /// Whether saving advancements is enabled.
    pub save_advancements: bool,
}

impl Default for AdvancementConfig {
    fn default() -> Self {
        Self {
            save_advancements: true,
        }
    }
}
