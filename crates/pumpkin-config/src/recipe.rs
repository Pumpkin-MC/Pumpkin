use serde::{Deserialize, Serialize};

/// Recipe-related configuration.
#[derive(Deserialize, Serialize, toml_spanner::Toml)]
#[serde(default)]
#[toml(ignore_unknown_fields, recoverable)]
pub struct RecipeConfig {
    /// Whether recipes are sent to clients, enabling the recipe book.
    #[toml(default = Self::default().send_recipes)]
    pub send_recipes: bool,
}

impl Default for RecipeConfig {
    fn default() -> Self {
        Self { send_recipes: true }
    }
}
