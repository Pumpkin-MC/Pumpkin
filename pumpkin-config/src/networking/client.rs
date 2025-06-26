use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct ClientConfig {
    /// Timeout in seconds for client connection handling. 0 disables timeout.
    pub connection_timeout: u32,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            connection_timeout: 30,
        }
    }
}
