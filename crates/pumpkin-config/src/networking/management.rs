use std::net::{Ipv4Addr, SocketAddr};

use rand::RngExt;
use serde::{Deserialize, Serialize};

const SECRET_CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

/// Configuration for the Minecraft Server Management Protocol (MSMP) service.
///
/// Controls whether the management server is enabled, its network address,
/// authentication secret, allowed origins, and heartbeat interval.
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(default)]
pub struct ManagementServerConfig {
    /// Whether the management server is enabled.
    pub enabled: bool,
    /// The network address and port where the management server will listen for WebSocket connections.
    pub address: SocketAddr,
    /// The 40-character alphanumeric secret required for authentication.
    /// If empty on startup, a random 40-character secret will be generated.
    pub secret: String,
    /// Allowed origins for Web browser connections via the `Origin` header.
    pub allowed_origins: Vec<String>,
    /// Interval in seconds between server status heartbeats (0 disables heartbeats).
    pub status_heartbeat_interval: u32,
}

impl Default for ManagementServerConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            address: SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 25585),
            secret: String::new(),
            allowed_origins: Vec::new(),
            status_heartbeat_interval: 0,
        }
    }
}

impl ManagementServerConfig {
    /// Generates a random 40-character alphanumeric secret key according to vanilla MSMP specification.
    #[must_use]
    pub fn generate_secret() -> String {
        let mut rng = rand::rng();
        (0..40)
            .map(|_| {
                let idx = rng.random_range(0..SECRET_CHARSET.len());
                SECRET_CHARSET[idx] as char
            })
            .collect()
    }

    /// Validates whether a secret key matches the expected 40-character alphanumeric format.
    #[must_use]
    pub fn is_valid_secret(secret: &str) -> bool {
        secret.len() == 40 && secret.chars().all(|c| c.is_ascii_alphanumeric())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_valid_secret() {
        let secret = ManagementServerConfig::generate_secret();
        assert_eq!(secret.len(), 40);
        assert!(ManagementServerConfig::is_valid_secret(&secret));
    }

    #[test]
    fn validates_invalid_secret_length_and_chars() {
        assert!(!ManagementServerConfig::is_valid_secret("too_short"));
        assert!(!ManagementServerConfig::is_valid_secret(&"a".repeat(41)));
        assert!(!ManagementServerConfig::is_valid_secret(&format!(
            "{}!",
            "a".repeat(39)
        )));
    }
}
