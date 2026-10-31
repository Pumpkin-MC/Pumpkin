use std::net::{Ipv4Addr, SocketAddr};

use serde::{Deserialize, Serialize};

/// Configuration for the server query protocol (legacy Minecraft query).
///
/// Controls whether the query service is enabled and which address it binds to.
#[derive(Deserialize, Serialize, toml_spanner::Toml)]
#[serde(default)]
#[toml(ignore_unknown_fields, recoverable)]
pub struct QueryConfig {
    /// Whether the query protocol is enabled.
    #[toml(default = Self::default().enabled)]
    pub enabled: bool,
    /// The address and port the query service binds to.
    #[toml(default = Self::default().address, with = toml_spanner::helper::parse_string)]
    pub address: SocketAddr,
}

impl Default for QueryConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            address: SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 25565),
        }
    }
}
