use pumpkin_macros::{Event, cancellable};
use pumpkin_util::text::TextComponent;
use std::net::SocketAddr;
use uuid::Uuid;

/// An asynchronous event that occurs when a connection attempts to pre-login.
#[cancellable]
#[derive(Event, Clone)]
pub struct AsyncPlayerPreLoginEvent {
    /// The player username.
    pub player_name: String,

    /// The player unique ID.
    pub player_uuid: Uuid,

    /// The remote IP address.
    pub ip_address: SocketAddr,

    /// The kick message if the connection is rejected.
    pub kick_message: TextComponent,
}
