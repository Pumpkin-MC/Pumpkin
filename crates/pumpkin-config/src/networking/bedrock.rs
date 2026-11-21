use crate::{CompressionConfig, PacketLimiterConfig};
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, SocketAddr};
use std::num::NonZero;
use std::path::PathBuf;

/// Configuration for Bedrock authentication.
#[derive(Deserialize, Serialize, Clone, toml_spanner::Toml)]
#[serde(default)]
#[toml(ignore_unknown_fields, recoverable)]
pub struct BedrockAuthenticationConfig {
    /// Whether Xbox Live authentication is enabled/enforced.
    #[toml(default = Self::default().enabled)]
    pub enabled: bool,
    /// Optional custom authentication/discovery URL.
    #[toml(default = Self::default().url)]
    pub url: Option<String>,
    /// Connection timeout in milliseconds.
    #[toml(default = Self::default().connect_timeout)]
    pub connect_timeout: u32,
    /// Read timeout in milliseconds.
    #[toml(default = Self::default().read_timeout)]
    pub read_timeout: u32,
}

/// Configuration for Bedrock's HTTP/WebRTC `NetherNet` transport.
#[derive(Deserialize, Serialize, Clone, toml_spanner::Toml)]
#[serde(default)]
#[toml(ignore_unknown_fields, recoverable)]
pub struct NetherNetConfig {
    /// Whether clients may connect using `NetherNet`.
    #[toml(default = Self::default().enabled)]
    pub enabled: bool,
    /// TCP signaling and shared UDP status/ICE address.
    #[toml(default = Self::default().address, with = toml_spanner::helper::parse_string)]
    pub address: SocketAddr,
    /// Optional public IP advertised when the ICE address is behind NAT.
    #[serde(with = "optional_ip")]
    #[toml(default = Self::default().external_ip, with = optional_ip)]
    pub external_ip: Option<IpAddr>,
    /// PKCS#8 P-384 identity key retained across restarts for Trust On First Use.
    #[toml(default = Self::default().identity_key)]
    pub identity_key: PathBuf,
    /// Optional ICE server URLs. Use `external_ip` for NAT with the single-port UDP mux.
    #[toml(default = Self::default().stun_servers)]
    pub stun_servers: Vec<String>,
}

mod optional_ip {
    pub fn from_toml<'de>(
        ctx: &mut toml_spanner::Context<'de>,
        item: &toml_spanner::Item<'de>,
    ) -> Result<Option<std::net::IpAddr>, toml_spanner::Failed> {
        let value = item.require_string(ctx)?.trim();
        if value.is_empty() {
            Ok(None)
        } else {
            value
                .parse()
                .map(Some)
                .map_err(|error| ctx.report_custom_error(error, item))
        }
    }

    use serde::{Deserialize, Deserializer, Serializer, de};
    use std::net::IpAddr;

    #[expect(
        clippy::ref_option,
        reason = "serde passes the configured field by reference"
    )]
    pub fn serialize<S: Serializer>(ip: &Option<IpAddr>, serializer: S) -> Result<S::Ok, S::Error> {
        match ip {
            Some(ip) => serializer.collect_str(ip),
            None => serializer.serialize_str(""),
        }
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<IpAddr>, D::Error> {
        let value = String::deserialize(deserializer)?;
        let value = value.trim();
        if value.is_empty() {
            Ok(None)
        } else {
            value.parse().map(Some).map_err(de::Error::custom)
        }
    }
}

impl Default for NetherNetConfig {
    fn default() -> Self {
        let address = "0.0.0.0:19132"
            .parse()
            .unwrap_or_else(|_| std::net::SocketAddr::from(([0, 0, 0, 0], 19132)));
        Self {
            enabled: true,
            address,
            external_ip: None,
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

#[cfg(test)]
mod tests {
    use super::NetherNetConfig;

    #[test]
    fn empty_external_ip_uses_automatic_detection() {
        let config: NetherNetConfig = toml::from_str("external_ip = \"\"").unwrap();
        assert_eq!(config.external_ip, None);
        assert!(
            toml::to_string(&config)
                .unwrap()
                .contains("external_ip = \"\"")
        );
    }

    #[test]
    fn explicit_external_ip_is_preserved() {
        let config: NetherNetConfig = toml::from_str("external_ip = \"203.0.113.7\"").unwrap();
        assert_eq!(config.external_ip.unwrap().to_string(), "203.0.113.7");
    }
}

/// Configuration for Bedrock Edition client connections.
#[derive(Deserialize, Serialize, Clone, toml_spanner::Toml)]
#[serde(default)]
#[toml(ignore_unknown_fields, recoverable)]
pub struct BedrockConfig {
    /// Whether Bedrock Edition Clients are Accepted.
    #[toml(default = Self::default().enabled)]
    pub enabled: bool,
    /// Whether online mode is enabled.
    #[toml(default = Self::default().online_mode)]
    pub online_mode: bool,
    /// The maximum number of players allowed on the server. Specifying `0` disables the limit.
    #[toml(default = Self::default().max_players)]
    pub max_players: u32,
    /// The maximum view distance for players.
    #[toml(default = Self::default().view_distance, with = crate::toml_adapters::serde_value)]
    pub view_distance: NonZero<u8>,
    /// The maximum simulated view distance.
    #[toml(default = Self::default().simulation_distance, with = crate::toml_adapters::serde_value)]
    pub simulation_distance: NonZero<u8>,
    /// Bedrock Edition packet compression settings.
    #[toml(default = Self::default().compression)]
    pub compression: CompressionConfig,
    /// Message of the Day; the server's description displayed on the status screen.
    #[toml(default = Self::default().motd)]
    pub motd: String,
    /// Prefix prepended to Bedrock Edition player names, so they cannot collide with
    /// Java Edition account names on a cross-play server. Empty means no prefix.
    #[toml(default = Self::default().username_prefix)]
    pub username_prefix: String,
    /// Whether spaces in Bedrock Edition player names are replaced with underscores.
    /// Names containing spaces cannot be typed as command arguments.
    #[toml(default = Self::default().replace_username_spaces)]
    pub replace_username_spaces: bool,
    /// Bedrock Edition authentication settings.
    #[toml(default = Self::default().authentication)]
    pub authentication: BedrockAuthenticationConfig,
    /// Bedrock `NetherNet` transport settings.
    #[toml(default = Self::default().nethernet)]
    pub nethernet: NetherNetConfig,
    /// Whether Bedrock client chunk blob caching is enabled.
    #[toml(default = Self::default().chunk_caching)]
    pub chunk_caching: bool,
    /// Packet rate limiting settings.
    #[toml(default = Self::default().packet_limiter)]
    pub packet_limiter: PacketLimiterConfig,
}

impl Default for BedrockConfig {
    fn default() -> Self {
        let view_distance = NonZero::new(16).unwrap_or(NonZero::<u8>::MIN);
        let simulation_distance = NonZero::new(10).unwrap_or(NonZero::<u8>::MIN);
        Self {
            enabled: true,
            online_mode: true,
            max_players: 1000,
            view_distance,
            simulation_distance,
            compression: CompressionConfig::default(),
            motd: "A blazingly fast Pumpkin server!".to_string(),
            username_prefix: String::new(),
            replace_username_spaces: true,
            authentication: BedrockAuthenticationConfig::default(),
            nethernet: NetherNetConfig::default(),
            chunk_caching: true,
            packet_limiter: PacketLimiterConfig::default(),
        }
    }
}
