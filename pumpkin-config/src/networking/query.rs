use std::net::{Ipv4Addr, SocketAddr};

use serde::{Deserialize, Serialize};

/// Configuration for the server query protocol (legacy Minecraft query).
///
/// Controls whether the query service is enabled and which address it binds to.
#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct QueryConfig {
    /// Whether the query protocol is enabled.
    pub enabled: bool,

    /// The address and port the query service binds to.
    pub address: SocketAddr,
}

impl Default for QueryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            address: SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 25565),
        }
    }
}
