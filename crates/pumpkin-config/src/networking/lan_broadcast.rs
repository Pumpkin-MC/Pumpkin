use serde::{Deserialize, Serialize};

/// Configuration for LAN broadcast of the server.
///
/// Controls whether the server is discoverable on the local network, and optional MOTD and port settings.
#[derive(Deserialize, Serialize, Default)]
#[serde(default)]
pub struct LANBroadcastConfig {
    /// Whether LAN broadcast is enabled.
    pub enabled: bool,
    /// Optional one-line Message of the Day (MOTD) for LAN clients.
    /// Defaults to the server MOTD with newlines removed.
    pub motd: Option<String>,
    /// Optional port for LAN broadcast.
    /// Useful for predictable ports in environments like Docker containers.
    pub port: Option<u16>,
}
