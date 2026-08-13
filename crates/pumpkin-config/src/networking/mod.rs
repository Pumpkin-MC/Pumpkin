use proxy::ProxyConfig;
use query::QueryConfig;
use rcon::RCONConfig;
use serde::{Deserialize, Serialize};

use crate::LANBroadcastConfig;
use bedrock::BedrockConfig;
use java::JavaConfig;

/// Authentication configuration.
pub mod auth;
/// Bedrock protocol networking configuration.
pub mod bedrock;
/// Packet compression configuration.
pub mod compression;
/// Java protocol networking configuration.
pub mod java;
/// LAN broadcast discovery configuration.
pub mod lan_broadcast;
/// Reverse proxy and BungeeCord/Velocity configuration.
pub mod proxy;
/// GS4 Query protocol configuration.
pub mod query;
/// RCON remote console configuration.
pub mod rcon;

/// Configuration for server networking features.
///
/// Covers authentication, query, RCON, proxying, packet compression,
/// and LAN broadcast behaviour.
#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct NetworkingConfig {
    /// How long in seconds a client may stay silent during the login sequence
    /// before it is disconnected.
    ///
    /// This is an inactivity timeout, not a deadline for the whole sequence:
    /// the timer resets whenever a packet arrives, so a client that is slow but
    /// still talking (for example downloading a large resource pack) is never
    /// kicked. Set to `0` to disable it.
    pub login_idle_timeout: u64,
    /// Query protocol settings for server status requests.
    pub query: QueryConfig,
    /// RCON (remote console) configuration.
    pub rcon: RCONConfig,
    /// Proxy-related networking settings.
    pub proxy: ProxyConfig,
    /// LAN broadcast settings.
    pub lan_broadcast: LANBroadcastConfig,
    /// Java Edition configuration settings.
    pub java: JavaConfig,
    /// Bedrock Edition configuration settings.
    pub bedrock: BedrockConfig,
}

impl Default for NetworkingConfig {
    fn default() -> Self {
        Self {
            login_idle_timeout: 60,
            query: QueryConfig::default(),
            rcon: RCONConfig::default(),
            proxy: ProxyConfig::default(),
            lan_broadcast: LANBroadcastConfig::default(),
            java: JavaConfig::default(),
            bedrock: BedrockConfig::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::NetworkingConfig;

    #[test]
    fn login_idle_timeout_defaults_to_one_minute() {
        assert_eq!(NetworkingConfig::default().login_idle_timeout, 60);
    }
}
