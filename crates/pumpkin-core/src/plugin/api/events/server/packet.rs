use bytes::Bytes;
use pumpkin_macros::{Event, cancellable};
use pumpkin_protocol::ConnectionState;
use pumpkin_util::version::{BedrockMinecraftVersion, JavaMinecraftVersion};
use std::sync::Arc;

use crate::entity::player::Player;

#[cancellable]
#[derive(Event, Clone)]
pub struct PacketReceivedEvent {
    pub player: Arc<Player>,
    pub packet_id: i32,
    pub payload: Bytes,
}

impl PacketReceivedEvent {
    pub const fn new(player: Arc<Player>, packet_id: i32, payload: Bytes) -> Self {
        Self {
            player,
            packet_id,
            payload,
            cancelled: false,
        }
    }
}

#[cancellable]
#[derive(Event, Clone)]
pub struct PacketSentEvent {
    pub player: Arc<Player>,
    pub packet_id: i32,
    pub payload: Bytes,
    pub packet: Arc<dyn std::any::Any + Send + Sync>,
    pub extra_packets: Vec<(i32, Bytes)>,
}

impl PacketSentEvent {
    pub fn new(
        player: Arc<Player>,
        packet_id: i32,
        payload: Bytes,
        packet: Arc<dyn std::any::Any + Send + Sync>,
    ) -> Self {
        Self {
            player,
            packet_id,
            payload,
            packet,
            extra_packets: Vec::new(),
            cancelled: false,
        }
    }

    /// For already serialized packets. `packet` is a placeholder (WIT needs one).
    pub fn new_raw(player: Arc<Player>, packet_id: i32, payload: Bytes) -> Self {
        struct RawPacket;
        Self::new(player, packet_id, payload, Arc::new(RawPacket))
    }
}

/// A Java packet from a client below `CURRENT_MC_VERSION` before play, in the client's format.
#[cancellable]
#[derive(Event, Clone)]
pub struct JavaConnectionPacketReceivedEvent {
    pub connection_id: u64,
    pub version: JavaMinecraftVersion,
    pub state: ConnectionState,
    pub packet_id: i32,
    pub payload: Bytes,
}

impl JavaConnectionPacketReceivedEvent {
    pub const fn new(
        connection_id: u64,
        version: JavaMinecraftVersion,
        state: ConnectionState,
        packet_id: i32,
        payload: Bytes,
    ) -> Self {
        Self {
            connection_id,
            version,
            state,
            packet_id,
            payload,
            cancelled: false,
        }
    }
}

/// A Java packet to a client below `CURRENT_MC_VERSION` before play, in the 26.3 format.
#[cancellable]
#[derive(Event, Clone)]
pub struct JavaConnectionPacketSentEvent {
    pub connection_id: u64,
    pub version: JavaMinecraftVersion,
    pub state: ConnectionState,
    pub packet_id: i32,
    pub payload: Bytes,
    pub extra_packets: Vec<(i32, Bytes)>,
}

impl JavaConnectionPacketSentEvent {
    pub const fn new(
        connection_id: u64,
        version: JavaMinecraftVersion,
        state: ConnectionState,
        packet_id: i32,
        payload: Bytes,
    ) -> Self {
        Self {
            connection_id,
            version,
            state,
            packet_id,
            payload,
            extra_packets: Vec::new(),
            cancelled: false,
        }
    }
}

/// A Bedrock packet from a client on a non-current protocol before play, in the client's format.
#[cancellable]
#[derive(Event, Clone)]
pub struct BedrockConnectionPacketReceivedEvent {
    pub connection_id: u64,
    pub protocol_version: u32,
    pub version: BedrockMinecraftVersion,
    pub packet_id: i32,
    pub payload: Bytes,
}

impl BedrockConnectionPacketReceivedEvent {
    pub const fn new(
        connection_id: u64,
        protocol_version: u32,
        version: BedrockMinecraftVersion,
        packet_id: i32,
        payload: Bytes,
    ) -> Self {
        Self {
            connection_id,
            protocol_version,
            version,
            packet_id,
            payload,
            cancelled: false,
        }
    }
}

/// A Bedrock packet to a client on a non-current protocol before play, in the current server format.
#[cancellable]
#[derive(Event, Clone)]
pub struct BedrockConnectionPacketSentEvent {
    pub connection_id: u64,
    pub protocol_version: u32,
    pub version: BedrockMinecraftVersion,
    pub packet_id: i32,
    pub payload: Bytes,
    pub extra_packets: Vec<(i32, Bytes)>,
}

impl BedrockConnectionPacketSentEvent {
    pub const fn new(
        connection_id: u64,
        protocol_version: u32,
        version: BedrockMinecraftVersion,
        packet_id: i32,
        payload: Bytes,
    ) -> Self {
        Self {
            connection_id,
            protocol_version,
            version,
            packet_id,
            payload,
            extra_packets: Vec::new(),
            cancelled: false,
        }
    }
}
