use bytes::Bytes;
use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

use crate::entity::player::Player;

/// Packet direction as seen by the server.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacketDirection {
    Serverbound,
    Clientbound,
}

/// Java connection state for packet hooks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JavaConnectionState {
    Handshake,
    Status,
    Login,
    Config,
    Play,
    Transfer,
}

/// Bedrock connection state for packet hooks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BedrockConnectionState {
    Offline,
    Raknet,
    Game,
}

/// Unified connection state for packet hooks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacketConnectionState {
    Java(JavaConnectionState),
    Bedrock(BedrockConnectionState),
}

/// Raw packet payload with an explicit packet ID.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawPacketData {
    pub id: i32,
    pub payload: Bytes,
}

/// A raw packet event that can be modified or cancelled by plugins.
#[cancellable]
#[derive(Event, Clone)]
pub struct RawPacketEvent {
    /// The player associated with this packet, if available.
    pub player: Option<Arc<Player>>,
    /// Packet direction relative to the server.
    pub direction: PacketDirection,
    /// Connection state when the packet was observed.
    pub state: PacketConnectionState,
    /// Raw packet data (ID + payload).
    pub packet: RawPacketData,
}

impl RawPacketEvent {
    #[must_use]
    pub const fn new(
        player: Option<Arc<Player>>,
        direction: PacketDirection,
        state: PacketConnectionState,
        packet: RawPacketData,
    ) -> Self {
        Self {
            player,
            direction,
            state,
            packet,
            cancelled: false,
        }
    }
}
