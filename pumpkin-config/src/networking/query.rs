use std::net::{Ipv4Addr, SocketAddr};

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct QueryConfig {
    pub enabled: bool,
    pub address: SocketAddr,
}

impl Default for QueryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            address: SocketAddr::new(Ipv4Addr::new(0, 0, 0, 0).into(), 25565),
        }
    }
}
