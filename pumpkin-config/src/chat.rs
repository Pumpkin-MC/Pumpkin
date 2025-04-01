use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct ChatConfig {
    /// The custom chat format (doesn't work with secure chat)
    pub format: String,
}

impl Default for ChatConfig {
    fn default() -> Self {
        Self {
            format: "<{DISPLAYNAME}> {MESSAGE}".to_string(),
        }
    }
}
