use crate::{AuthenticationConfig, CompressionConfig, PacketLimiterConfig};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::num::NonZero;

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
    pub view_distance: NonZero<u8>,
    /// The maximum simulated view distance.
    pub simulation_distance: NonZero<u8>,
    /// Time interval in seconds between keep-alive packets sent to Java clients.
    #[serde(
        alias = "keep-alive-time",
        alias = "keep_alive_interval",
        alias = "keep-alive-interval"
    )]
    pub keep_alive_time: u64,
    /// Java Edition packet compression settings.
    pub compression: CompressionConfig,
    /// Message of the Day; the server's description displayed on the status screen.
    pub motd: String,
    /// Authentication settings for client connections.
    pub authentication: AuthenticationConfig,
    /// Packet rate limiting settings.
    pub packet_limiter: PacketLimiterConfig,
}

impl Default for JavaConfig {
    fn default() -> Self {
        let address = "0.0.0.0:25565"
            .parse()
            .unwrap_or_else(|_| std::net::SocketAddr::from(([0, 0, 0, 0], 25565)));
        let view_distance = NonZero::new(16).unwrap_or(NonZero::<u8>::MIN);
        let simulation_distance = NonZero::new(10).unwrap_or(NonZero::<u8>::MIN);
        Self {
            enabled: true,
            address,
            encryption: true,
            online_mode: true,
            max_players: 1000,
            view_distance,
            simulation_distance,
            keep_alive_time: 15,
            compression: CompressionConfig::default(),
            motd: "A blazingly fast Pumpkin server!".to_string(),
            authentication: AuthenticationConfig::default(),
            packet_limiter: PacketLimiterConfig::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_keep_alive_time() {
        let config = JavaConfig::default();
        assert_eq!(config.keep_alive_time, 15);
    }

    #[test]
    fn keep_alive_time_deserialization() {
        let toml_snake = r"
            keep_alive_time = 20
        ";
        let config: JavaConfig = toml::from_str(toml_snake).unwrap();
        assert_eq!(config.keep_alive_time, 20);

        let toml_kebab = r"
            keep-alive-time = 25
        ";
        let config: JavaConfig = toml::from_str(toml_kebab).unwrap();
        assert_eq!(config.keep_alive_time, 25);

        let toml_interval = r"
            keep_alive_interval = 30
        ";
        let config: JavaConfig = toml::from_str(toml_interval).unwrap();
        assert_eq!(config.keep_alive_time, 30);

        let toml_interval_kebab = r"
            keep-alive-interval = 35
        ";
        let config: JavaConfig = toml::from_str(toml_interval_kebab).unwrap();
        assert_eq!(config.keep_alive_time, 35);
    }
}
