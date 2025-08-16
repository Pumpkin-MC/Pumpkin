use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default)]
pub struct NetworksConfig {
    /// Enable IPv6 support with dual-stack binding (falls back to IPv4 if unavailable)
    pub ipv6_enabled: bool,
}
