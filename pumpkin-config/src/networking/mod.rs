use auth::AuthenticationConfig;
use proxy::ProxyConfig;
use query::QueryConfig;
use rcon::RCONConfig;
use serde::{Deserialize, Serialize};

use crate::{CompressionConfig, LANBroadcastConfig};

pub mod auth;
pub mod compression;
pub mod lan_broadcast;
pub mod proxy;
pub mod query;
pub mod rcon;

/// Configuration for server networking features.
///
/// Covers authentication, query, RCON, proxying, packet compression,
/// and LAN broadcast behaviour.
#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct NetworkingConfig {
    /// Maximum time, in milliseconds, that a client may remain in the login/configuration flow
    /// before being disconnected. Set to `0` to disable this timeout.
    pub login_timeout: u32,
    /// Authentication settings for client connections.
    pub authentication: AuthenticationConfig,
    /// Query protocol settings for server status requests.
    pub query: QueryConfig,
    /// RCON (remote console) configuration.
    pub rcon: RCONConfig,
    /// Proxy-related networking settings.
    pub proxy: ProxyConfig,
    /// Packet compression settings.
    pub packet_compression: CompressionConfig,
    /// LAN broadcast settings.
    pub lan_broadcast: LANBroadcastConfig,
}

const fn default_login_timeout() -> u32 {
    30_000
}

impl Default for NetworkingConfig {
    fn default() -> Self {
        Self {
            login_timeout: default_login_timeout(),
            authentication: AuthenticationConfig::default(),
            query: QueryConfig::default(),
            rcon: RCONConfig::default(),
            proxy: ProxyConfig::default(),
            packet_compression: CompressionConfig::default(),
            lan_broadcast: LANBroadcastConfig::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::NetworkingConfig;

    #[test]
    fn login_timeout_defaults_to_30_seconds() {
        assert_eq!(NetworkingConfig::default().login_timeout, 30_000);
    }

    #[test]
    fn login_timeout_is_filled_by_serde_default() {
        let config: NetworkingConfig = toml::from_str("").expect("valid empty config");

        assert_eq!(config.login_timeout, 30_000);
    }
}
