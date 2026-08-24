use serde::{Deserialize, Serialize};

/// Lighting engine calculation mode.
#[derive(Deserialize, Serialize, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum LightingEngineConfig {
    /// Default Vanilla lighting propagation.
    #[default]
    Default,
    /// Full skylight everywhere (no shadows).
    Full,
    /// Completely dark lighting everywhere (zero light).
    Dark,
}
