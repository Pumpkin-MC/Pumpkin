use serde::{Deserialize, Serialize};
use std::net::{Ipv4Addr, SocketAddr};

/// Configuration for the RCON (Remote Console) service.
///
/// Controls whether RCON is enabled, connection settings, authentication, and logging.
#[derive(Deserialize, Serialize, Clone)]
#[serde(default)]
pub struct RCONConfig {
    /// Whether RCON is enabled.
    pub enabled: bool,
    /// The network address and port where the RCON server will listen for connections.
    pub address: SocketAddr,
    /// The password required for RCON authentication.
    pub password: String,
    /// The maximum number of concurrent RCON connections allowed.
    /// A value of `0` indicates no limit.
    pub max_connections: u32,
    /// Logging configuration for RCON events.
    pub logging: RCONLogging,
}

impl Default for RCONConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            address: SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 25575),
            password: String::new(),
            max_connections: 0,
            logging: RCONLogging::default(),
        }
    }
}

/// Logging settings for RCON.
///
/// Controls which RCON events are logged, including login attempts and commands.
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(default)]
pub struct RCONLogging {
    /// Whether successful RCON logins should be logged.
    pub logged_successfully: bool,
    /// Whether failed RCON login attempts with incorrect passwords should be logged.
    pub wrong_password: bool,
    /// Whether all RCON commands, regardless of success or failure, should be logged.
    pub commands: bool,
    /// Whether RCON quit commands should be logged.
    pub quit: bool,
}

impl Default for RCONLogging {
    fn default() -> Self {
        Self {
            logged_successfully: true,
            wrong_password: true,
            commands: true,
            quit: true,
        }
    }
}
