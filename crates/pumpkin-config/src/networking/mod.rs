use management::ManagementServerConfig;
use proxy::ProxyConfig;
use query::QueryConfig;
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
/// Management server (MSMP) configuration.
pub mod management;
/// Reverse proxy and BungeeCord/Velocity configuration.
pub mod proxy;
/// GS4 Query protocol configuration.
pub mod query;

/// Packet limiter configuration.
pub mod packet_limiter;
pub use packet_limiter::PacketLimiterConfig;

/// Configuration for server networking features.
///
/// Covers authentication, query, management protocol, proxying, packet compression,
/// and LAN broadcast behaviour.
#[derive(Deserialize, Serialize, Default, toml_spanner::Toml)]
#[serde(default)]
#[toml(ignore_unknown_fields, recoverable)]
pub struct NetworkingConfig {
    /// Query protocol settings for server status requests.
    #[toml(default = Self::default().query)]
    pub query: QueryConfig,
    /// Management server (MSMP) configuration.
    #[toml(default = Self::default().management)]
    pub management: ManagementServerConfig,
    /// Proxy-related networking settings.
    #[toml(default = Self::default().proxy)]
    pub proxy: ProxyConfig,
    /// LAN broadcast settings.
    #[toml(default = Self::default().lan_broadcast)]
    pub lan_broadcast: LANBroadcastConfig,
    /// Java Edition configuration settings.
    #[toml(default = Self::default().java)]
    pub java: JavaConfig,
    /// Bedrock Edition configuration settings.
    #[toml(default = Self::default().bedrock)]
    pub bedrock: BedrockConfig,
}
