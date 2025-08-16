use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct NetworksConfig {
    /// Enable IPv6 support with dual-stack binding (falls back to IPv4 if unavailable)
    pub ipv6_enabled: bool,
}

impl Default for NetworksConfig {
    fn default() -> Self {
        Self {
            ipv6_enabled: false,
        }
    }
}