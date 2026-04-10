use serde::{Deserialize, Serialize};

/// Recipe-related configuration.
#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct RecipeConfig {
    /// Whether recipes are sent to clients, enabling the recipe book.
    pub send_recipes: bool,
}

impl Default for RecipeConfig {
    fn default() -> Self {
        Self { send_recipes: true }
    }
}
