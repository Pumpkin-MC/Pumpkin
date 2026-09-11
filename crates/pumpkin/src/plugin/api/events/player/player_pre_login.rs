use pumpkin_macros::{Event, cancellable};
use pumpkin_util::text::TextComponent;
use std::net::SocketAddr;
use uuid::Uuid;

/// An event that occurs synchronously when a player pre-logins.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerPreLoginEvent {
    /// The player username.
    pub player_name: String,

    /// The player unique ID.
    pub player_uuid: Uuid,

    /// The remote IP address.
    pub ip_address: SocketAddr,

    /// The kick message if rejected.
    pub kick_message: TextComponent,
}
