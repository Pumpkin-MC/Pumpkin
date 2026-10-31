use serde::{Deserialize, Serialize};

/// Configuration for proxy support.
///
/// Allows integration with proxy servers like Velocity and `BungeeCord`.
#[derive(Deserialize, Serialize, Default, toml_spanner::Toml)]
#[serde(default)]
#[toml(ignore_unknown_fields, recoverable)]
pub struct ProxyConfig {
    /// Whether proxy support is enabled.
    #[toml(default = Self::default().enabled)]
    pub enabled: bool,
    /// Configuration for Velocity proxy integration.
    #[toml(default = Self::default().velocity)]
    pub velocity: VelocityConfig,
    /// Configuration for `BungeeCord` proxy integration.
    #[toml(default = Self::default().bungeecord)]
    pub bungeecord: BungeeCordConfig,
    /// Configuration for Vine modern proxy integration with Ed25519 authentication.
    #[toml(default = Self::default().vine)]
    pub vine: VineConfig,
}

/// Configuration for `BungeeCord` proxy integration.
#[derive(Deserialize, Serialize, Default, toml_spanner::Toml)]
#[serde(default)]
#[toml(ignore_unknown_fields, recoverable)]
pub struct BungeeCordConfig {
    /// Whether `BungeeCord` support is enabled.
    #[toml(default = Self::default().enabled)]
    pub enabled: bool,
    /// Shared secret for authenticating connections from the `BungeeCord`
    /// proxy, as provided by the `BungeeGuard` plugin. When set, the forwarded
    /// profile properties must contain a `bungeeguard-token` property holding
    /// this secret, otherwise the connection is rejected. This also blocks
    /// players connecting directly instead of through the proxy.
    #[toml(default = Self::default().secret)]
    pub secret: String,
}

/// Configuration for Velocity proxy integration.
#[derive(Deserialize, Serialize, Default, toml_spanner::Toml)]
#[serde(default)]
#[toml(ignore_unknown_fields, recoverable)]
pub struct VelocityConfig {
    /// Whether Velocity support is enabled.
    #[toml(default = Self::default().enabled)]
    pub enabled: bool,
    /// Shared secret for authenticating connections from the Velocity proxy.
    #[toml(default = Self::default().secret)]
    pub secret: String,
}

/// Configuration for Vine proxy integration with Ed25519 authentication and replay protection.
#[derive(Deserialize, Serialize, Default, Clone, toml_spanner::Toml)]
#[serde(default)]
#[toml(ignore_unknown_fields, recoverable)]
pub struct VineConfig {
    /// Whether Vine support is enabled.
    #[toml(default = Self::default().enabled)]
    pub enabled: bool,
    /// Ed25519 public key (64 hex characters) of the Vine proxy.
    /// Backend only needs this public key to verify forwarded player identities.
    #[toml(default = Self::default().public_key)]
    pub public_key: String,
    /// Optional shared secret string. If provided and `public_key` is empty,
    /// the public key is automatically derived from this secret.
    #[toml(default = Self::default().secret)]
    pub secret: String,
}
