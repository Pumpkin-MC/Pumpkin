use crate::CompressionConfig;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::num::NonZeroU8;
use std::path::PathBuf;

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

/// Configuration for Bedrock's HTTP/WebRTC `NetherNet` transport.
#[derive(Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct NetherNetConfig {
    /// Whether clients may connect using `NetherNet`.
    pub enabled: bool,
    /// HTTP signaling address. WebRTC uses separately allocated UDP ports for game traffic.
    pub address: SocketAddr,
    /// PKCS#8 P-384 identity key retained across restarts for Trust On First Use.
    pub identity_key: PathBuf,
    /// Optional STUN server URLs used to gather a public ICE candidate behind NAT.
    pub stun_servers: Vec<String>,
}

impl Default for NetherNetConfig {
    fn default() -> Self {
        let address = "0.0.0.0:19132"
            .parse()
            .unwrap_or_else(|_| std::net::SocketAddr::from(([0, 0, 0, 0], 19132)));
        Self {
            enabled: true,
            address,
            identity_key: "nethernet-key.der".into(),
            stun_servers: Vec::new(),
        }
    }
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
    /// Bedrock `NetherNet` transport settings.
    pub nethernet: NetherNetConfig,
}

impl Default for BedrockConfig {
    fn default() -> Self {
        let view_distance = NonZeroU8::new(16).unwrap_or(NonZeroU8::MIN);
        let simulation_distance = NonZeroU8::new(10).unwrap_or(NonZeroU8::MIN);
        Self {
            enabled: true,
            online_mode: true,
            max_players: 1000,
            view_distance,
            simulation_distance,
            compression: CompressionConfig::default(),
            motd: "A blazingly fast Pumpkin server!".to_string(),
            authentication: BedrockAuthenticationConfig::default(),
            nethernet: NetherNetConfig::default(),
        }
    }
}
