use crate::{AuthenticationConfig, CompressionConfig, PacketLimiterConfig};
use ipnet::IpNet;
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, SocketAddr};
use std::num::NonZero;

/// Java TCP PROXY protocol v2 trust and header deadline settings.
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(default)]
pub struct ProxyProtocolConfig {
    /// Require a v2 header before Minecraft packets when enabled.
    pub enabled: bool,
    /// Networks allowed to supply headers, matched against the TCP peer.
    pub trusted_proxies: Vec<IpNet>,
    /// Overall header deadline in milliseconds, from 1 through 60000.
    pub header_timeout_ms: u64,
}

impl Default for ProxyProtocolConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            trusted_proxies: Vec::new(),
            header_timeout_ms: 5000,
        }
    }
}

impl ProxyProtocolConfig {
    /// Validate explicit trust and the finite header deadline.
    pub fn validate(&self) {
        assert!(
            !self.enabled || !self.trusted_proxies.is_empty(),
            "Java PROXY protocol requires explicit trusted_proxies"
        );
        assert!(
            self.header_timeout_ms > 0 && self.header_timeout_ms <= 60_000,
            "Java PROXY protocol header_timeout_ms must be between 1 and 60000"
        );
    }

    /// Match native addresses and the IPv4 form of IPv4-mapped IPv6 peers.
    #[must_use]
    pub fn trusts(&self, peer: IpAddr) -> bool {
        self.trusted_proxies.iter().any(|network| {
            network.contains(&peer)
                || peer.to_canonical() != peer && network.contains(&peer.to_canonical())
        })
    }
}

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
    /// Trusted `HAProxy` PROXY protocol v2 connections.
    pub proxy_protocol: ProxyProtocolConfig,
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
            proxy_protocol: ProxyProtocolConfig::default(),
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

#[cfg(test)]
mod proxy_protocol_tests {
    use super::*;
    use crate::PumpkinConfig;

    #[test]
    fn cidrs_and_mapped_peer_policy() {
        let config: ProxyProtocolConfig = toml::from_str(
            "trusted_proxies = ['192.0.2.0/24', '2001:db8::/32', '::ffff:198.51.100.0/120']",
        )
        .unwrap();
        for peer in [
            "192.0.2.255",
            "2001:db8::1",
            "::ffff:192.0.2.42",
            "::ffff:198.51.100.7",
        ] {
            assert!(config.trusts(peer.parse().unwrap()), "{peer}");
        }
        for peer in ["192.0.3.0", "2001:db9::1", "::192.0.2.42", "198.51.100.7"] {
            assert!(!config.trusts(peer.parse().unwrap()), "{peer}");
        }
        for cidr in ["192.0.2.1", "192.0.2.0/33", "::/129", "hostname/24"] {
            assert!(
                toml::from_str::<ProxyProtocolConfig>(&format!("trusted_proxies = ['{cidr}']"))
                    .is_err()
            );
        }
    }

    #[test]
    fn validation_and_generated_layout() {
        let mut config = ProxyProtocolConfig {
            enabled: true,
            ..Default::default()
        };
        assert!(std::panic::catch_unwind(|| config.validate()).is_err());
        config.trusted_proxies.push("127.0.0.1/32".parse().unwrap());
        config.validate();
        for timeout in [0, 60_001, u64::MAX] {
            config.header_timeout_ms = timeout;
            assert!(std::panic::catch_unwind(|| config.validate()).is_err());
        }
        let generated = toml::to_string(&PumpkinConfig::default()).unwrap();
        assert!(generated.contains("[networking.java.proxy_protocol]"));
        let value: toml::Value = toml::from_str(&generated).unwrap();
        assert_eq!(
            value["networking"]["java"]["proxy_protocol"]["header_timeout_ms"].as_integer(),
            Some(5000)
        );
        assert_eq!(
            value["networking"]["java"]["proxy_protocol"]["enabled"].as_bool(),
            Some(false)
        );
    }
}
