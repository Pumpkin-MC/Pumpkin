use serde::{Deserialize, Serialize};

/// Configuration for proxy support.
///
/// Allows integration with proxy servers like Velocity and `BungeeCord`.
#[derive(Deserialize, Serialize, Default)]
#[serde(default)]
pub struct ProxyConfig {
    /// Whether proxy support is enabled.
    pub enabled: bool,
    /// Configuration for Velocity proxy integration.
    pub velocity: VelocityConfig,
    /// Configuration for `BungeeCord` proxy integration.
    pub bungeecord: BungeeCordConfig,
}

/// Configuration for `BungeeCord` proxy integration.
#[derive(Deserialize, Serialize, Default)]
#[serde(default)]
pub struct BungeeCordConfig {
    /// Whether `BungeeCord` support is enabled.
    pub enabled: bool,
    /// Shared secret for `BungeeGuard` authentication.
    /// When set, incoming handshake data from `BungeeCord` must include a
    /// matching token. Leave empty if you are not using `BungeeGuard`.
    pub secret: String,
}

/// Configuration for Velocity proxy integration.
#[derive(Deserialize, Serialize, Default)]
#[serde(default)]
pub struct VelocityConfig {
    /// Whether Velocity support is enabled.
    pub enabled: bool,
    /// Shared secret for authenticating connections from the Velocity proxy.
    pub secret: String,
}
