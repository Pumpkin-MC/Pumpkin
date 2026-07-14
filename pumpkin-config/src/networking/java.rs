use crate::{AuthenticationConfig, CompressionConfig};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::num::NonZeroU8;

/// Configuration for Java Edition client connections.
#[derive(Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct JavaConfig {
    /// Whether Java Edition Clients are Accepted.
    pub enabled: bool,
    /// The address and port to which the Java Edition server will bind.
    pub address: SocketAddr,
    /// Whether packet encryption is enabled. Required when online mode is enabled.
    pub encryption: bool,
    /// Whether online mode is enabled. Requires valid Minecraft accounts.
    pub online_mode: bool,
    /// The maximum number of players allowed on the server. Specifying `0` disables the limit.
    pub max_players: u32,
    /// The maximum view distance for players.
    pub view_distance: NonZeroU8,
    /// The maximum simulated view distance.
    pub simulation_distance: NonZeroU8,
    /// Java Edition packet compression settings.
    pub compression: CompressionConfig,
    /// Message of the Day; the server's description displayed on the status screen.
    pub motd: String,
    /// Authentication settings for client connections.
    pub authentication: AuthenticationConfig,
}

impl Default for JavaConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            address: "0.0.0.0:25565".parse().unwrap(),
            encryption: true,
            online_mode: true,
            max_players: 1000,
            view_distance: NonZeroU8::new(16).unwrap(),
            simulation_distance: NonZeroU8::new(10).unwrap(),
            compression: CompressionConfig::default(),
            motd: "A blazingly fast Pumpkin server!".to_string(),
            authentication: AuthenticationConfig::default(),
        }
    }
}
