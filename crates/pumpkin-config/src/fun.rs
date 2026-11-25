use serde::{Deserialize, Serialize};

/// Let's face it, the only reason we play this game is because of fun. 🙃
#[derive(Deserialize, Serialize, toml_spanner::Toml)]
#[serde(default)]
#[toml(ignore_unknown_fields, recoverable)]
pub struct FunConfig {
    /// Whether April Fools features are enabled.
    #[toml(default = Self::default().april_fools)]
    pub april_fools: bool,
}

impl Default for FunConfig {
    fn default() -> Self {
        Self { april_fools: true }
    }
}
