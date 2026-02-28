use serde::{Deserialize, Serialize};

/// Configuration for in-game chat behaviour.
///
/// Controls chat formatting and display.
#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct ChatConfig {
    /// The custom chat format.
    /// `Note`: it does not apply when secure chat is enabled.
    pub format: String,
}

impl Default for ChatConfig {
    fn default() -> Self {
        Self {
            format: "<{DISPLAYNAME}> {MESSAGE}".to_string(),
        }
    }
}
