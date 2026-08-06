use crate::CompressionConfig;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::num::NonZeroU8;

/// Configuration for Bedrock authentication.
#[derive(Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct BedrockAuthenticationConfig {
    /// Whether Xbox Live authentication is enabled/enforced.
    pub enabled: bool,
    /// Optional custom authentication/discovery URL.
    pub url: Option<String>,
    /// Connection timeout in milliseconds.
    pub connect_timeout: u32,
    /// Read timeout in milliseconds.
    pub read_timeout: u32,
}

impl Default for BedrockAuthenticationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            url: None,
            connect_timeout: 5000,
            read_timeout: 5000,
        }
    }
}

/// Configuration for Bedrock Edition client connections.
#[derive(Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct BedrockConfig {
    /// Whether Bedrock Edition Clients are Accepted.
    pub enabled: bool,
    /// Whether Bedrock Edition Clients are Accepted.
    pub address: SocketAddr,
    /// Whether packet encryption is enabled for Bedrock Edition.
    pub encryption: bool,
    /// Whether online mode is enabled.
    pub online_mode: bool,
    /// The maximum number of players allowed on the server. Specifying `0` disables the limit.
    pub max_players: u32,
    /// The maximum view distance for players.
    pub view_distance: NonZeroU8,
    /// The maximum simulated view distance.
    pub simulation_distance: NonZeroU8,
    /// Bedrock Edition packet compression settings.
    pub compression: CompressionConfig,
    /// Message of the Day; the server's description displayed on the status screen.
    pub motd: String,
    /// Bedrock Edition authentication settings.
    pub authentication: BedrockAuthenticationConfig,
}

impl Default for BedrockConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            address: "0.0.0.0:19132".parse().unwrap(),
            encryption: true,
            online_mode: true,
            max_players: 1000,
            view_distance: NonZeroU8::new(16).unwrap(),
            simulation_distance: NonZeroU8::new(10).unwrap(),
            compression: CompressionConfig::default(),
            motd: "A blazingly fast Pumpkin server!".to_string(),
            authentication: BedrockAuthenticationConfig::default(),
        }
    }
}
