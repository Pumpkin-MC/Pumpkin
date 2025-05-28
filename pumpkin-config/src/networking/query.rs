use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default)]
#[serde(default)]
pub struct QueryConfig {
    pub enabled: bool,
    /// Optional; if not specified, the port the server is running on will be used.
    pub port: Option<u16>,
}
