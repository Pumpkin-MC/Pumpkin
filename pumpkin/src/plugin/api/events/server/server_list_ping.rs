use pumpkin_macros::Event;

/// An event that occurs when the server responds to a status ping.
#[derive(Event, Clone)]
pub struct ServerListPingEvent {
    /// The MOTD shown in the server list.
    pub motd: String,

    /// The maximum player count.
    pub max_players: u32,

    /// The current online player count.
    pub num_players: u32,

    /// The favicon as a data URI (if any).
    pub favicon: Option<String>,
}

impl ServerListPingEvent {
    /// Creates a new `ServerListPingEvent`.
    #[must_use]
    #[expect(clippy::missing_const_for_fn)]
    pub fn new(motd: String, max_players: u32, num_players: u32, favicon: Option<String>) -> Self {
        Self {
            motd,
            max_players,
            num_players,
            favicon,
        }
    }
}
