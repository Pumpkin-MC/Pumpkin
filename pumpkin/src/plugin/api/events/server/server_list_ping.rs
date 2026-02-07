use pumpkin_macros::{Event, cancellable};

/// An event that occurs when the server receives a status (ping) request from a client.
///
/// This allows plugins to customize the server list response: MOTD, max players,
/// online count, and protocol version name.
///
/// If cancelled, the status response is not sent.
///
/// Matches Bukkit's `ServerListPingEvent`.
#[cancellable]
#[derive(Event, Clone)]
pub struct ServerListPingEvent {
    /// The description (MOTD) shown in the server list.
    pub motd: String,

    /// Maximum number of players shown in the server list.
    pub max_players: u32,

    /// Current online player count shown in the server list.
    pub online_players: u32,

    /// The protocol version name shown in the server list (e.g. "1.21.11").
    pub version_name: String,

    /// The numeric protocol version.
    pub protocol_version: u32,
}

impl ServerListPingEvent {
    #[must_use]
    pub fn new(
        motd: String,
        max_players: u32,
        online_players: u32,
        version_name: String,
        protocol_version: u32,
    ) -> Self {
        Self {
            motd,
            max_players,
            online_players,
            version_name,
            protocol_version,
            cancelled: false,
        }
    }
}
