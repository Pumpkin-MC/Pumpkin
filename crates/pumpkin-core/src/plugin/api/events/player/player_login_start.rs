use pumpkin_macros::{Event, cancellable};
use pumpkin_util::text::TextComponent;
use std::net::SocketAddr;
use uuid::Uuid;

/// An event that occurs when a Java connection sends Login Start, before the server
/// decides whether to authenticate it with Mojang.
///
/// It only fires for direct connections. Behind a proxy, the proxy authenticates players.
///
/// `online_mode` starts as the configured `java.online_mode` and decides, for this
/// connection only, whether the player gets their online UUID, is asked to authenticate
/// and has their session checked with Mojang. Setting it to `true` also turns on
/// encryption for the connection, because authentication needs it.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerLoginStartEvent {
    /// The username sent by the client.
    pub player_name: String,

    /// The UUID sent by the client. It is not verified.
    pub player_uuid: Uuid,

    /// The remote IP address.
    pub ip_address: SocketAddr,

    /// The protocol version from the handshake.
    pub protocol_version: i32,

    /// Whether this connection is authenticated with Mojang.
    pub online_mode: bool,

    /// The kick message if the connection is rejected.
    pub kick_message: TextComponent,
}
