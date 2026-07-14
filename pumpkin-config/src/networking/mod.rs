use proxy::ProxyConfig;
use query::QueryConfig;
use rcon::RCONConfig;
use serde::{Deserialize, Serialize};

use crate::LANBroadcastConfig;
use bedrock::BedrockConfig;
use java::JavaConfig;

pub mod auth;
pub mod bedrock;
pub mod compression;
pub mod java;
pub mod lan_broadcast;
pub mod proxy;
pub mod query;
pub mod rcon;

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
