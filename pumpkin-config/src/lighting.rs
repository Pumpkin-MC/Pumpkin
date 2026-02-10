use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum LightingEngineConfig {
    #[default]
    Default,
    Full,
    Dark,
}
