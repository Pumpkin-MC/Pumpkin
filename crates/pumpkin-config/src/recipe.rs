use serde::{Deserialize, Serialize};

/// Recipe-related configuration.
#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct RecipeConfig {
    /// Whether the recipe book packets are sent when a player joins. The stonecutter recipes are
    /// always sent, because the stonecutter does not work without them.
    pub send_recipes: bool,
}

impl Default for RecipeConfig {
    fn default() -> Self {
        Self { send_recipes: true }
    }
}
