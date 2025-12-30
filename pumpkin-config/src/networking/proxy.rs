use std::net::IpAddr;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default)]
#[serde(default)]
pub struct ProxyConfig {
    pub enabled: bool,
    pub velocity: VelocityConfig,
    pub bungeecord: BungeeCordConfig,
}

#[derive(Deserialize, Serialize, Default)]
#[serde(default)]
pub struct BungeeCordConfig {
    pub enabled: bool,
    /// List of allowed proxy IP addresses. If empty, all IPs are allowed.
    /// Only connections from these IPs will be accepted when BungeeCord is enabled.
    #[serde(default)]
    pub allowed_ips: Vec<IpAddr>,
}

#[derive(Deserialize, Serialize, Default)]
#[serde(default)]
pub struct VelocityConfig {
    pub enabled: bool,
    pub secret: String,
}
