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
    /// Configuration for Vine modern proxy integration with Ed25519 authentication.
    pub vine: VineConfig,
}

/// Configuration for `BungeeCord` proxy integration.
#[derive(Deserialize, Serialize, Default)]
#[serde(default)]
pub struct BungeeCordConfig {
    /// Whether `BungeeCord` support is enabled.
    pub enabled: bool,
    /// Shared secret for authenticating connections from the `BungeeCord`
    /// proxy, as provided by the `BungeeGuard` plugin. When set, the forwarded
    /// profile properties must contain a `bungeeguard-token` property holding
    /// this secret, otherwise the connection is rejected. This also blocks
    /// players connecting directly instead of through the proxy.
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

/// Configuration for Vine proxy integration with Ed25519 authentication and replay protection.
#[derive(Deserialize, Serialize, Default, Clone)]
#[serde(default)]
pub struct VineConfig {
    /// Whether Vine support is enabled.
    pub enabled: bool,
    /// Ed25519 public key (64 hex characters) of the Vine proxy.
    /// Backend only needs this public key to verify forwarded player identities.
    pub public_key: String,
    /// Optional shared secret string. If provided and `public_key` is empty,
    /// the public key is automatically derived from this secret.
    pub secret: String,
}
