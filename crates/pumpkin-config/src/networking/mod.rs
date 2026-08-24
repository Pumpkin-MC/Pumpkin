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

/// Packet limiter configuration.
pub mod packet_limiter;
pub use packet_limiter::PacketLimiterConfig;

/// Configuration for server networking features.
///
/// Covers authentication, query, RCON, proxying, packet compression,
/// and LAN broadcast behaviour.
#[derive(Deserialize, Serialize, Default)]
#[serde(default)]
pub struct NetworkingConfig {
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
