use serde::{Deserialize, Serialize};

/// Let's face it, the only reason we play this game is because of fun. 🙃
#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct FunConfig {
    /// Whether April Fools features are enabled.
    pub april_fools: bool,
}

impl Default for FunConfig {
    fn default() -> Self {
        Self { april_fools: true }
    }
}
