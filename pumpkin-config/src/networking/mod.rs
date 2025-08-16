use auth::AuthenticationConfig;
use networks::NetworksConfig;
use proxy::ProxyConfig;
use query::QueryConfig;
use rcon::RCONConfig;
use serde::{Deserialize, Serialize};

use crate::{CompressionConfig, LANBroadcastConfig};

pub mod auth;
pub mod compression;
pub mod lan_broadcast;
pub mod networks;
pub mod proxy;
pub mod query;
pub mod rcon;

#[derive(Deserialize, Serialize, Default)]
pub struct NetworkingConfig {
    pub authentication: AuthenticationConfig,
    pub query: QueryConfig,
    pub rcon: RCONConfig,
    pub proxy: ProxyConfig,
    pub packet_compression: CompressionConfig,
    pub lan_broadcast: LANBroadcastConfig,
    pub networks: NetworksConfig,
}
