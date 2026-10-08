use bytes::Bytes;
use pumpkin_macros::{Event, cancellable};
use pumpkin_protocol::ConnectionState;
use pumpkin_util::version::JavaMinecraftVersion;
use std::sync::Arc;

use crate::entity::player::Player;
use crate::net::java::features::JavaConnectionFeatures;

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
    /// Java only: id + payload in the client's format, sent right after this packet.
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
pub struct ConnectionPacketReceivedEvent {
    pub connection_id: u64,
    pub version: JavaMinecraftVersion,
    pub state: ConnectionState,
    pub packet_id: i32,
    pub payload: Bytes,
    /// Kept for the connection.
    pub features: JavaConnectionFeatures,
}

impl ConnectionPacketReceivedEvent {
    pub const fn new(
        connection_id: u64,
        version: JavaMinecraftVersion,
        state: ConnectionState,
        packet_id: i32,
        payload: Bytes,
        features: JavaConnectionFeatures,
    ) -> Self {
        Self {
            connection_id,
            version,
            state,
            packet_id,
            payload,
            features,
            cancelled: false,
        }
    }
}

/// A Java packet to a client below `CURRENT_MC_VERSION` before play, in the 26.3 format.
#[cancellable]
#[derive(Event, Clone)]
pub struct ConnectionPacketSentEvent {
    pub connection_id: u64,
    pub version: JavaMinecraftVersion,
    pub state: ConnectionState,
    pub packet_id: i32,
    pub payload: Bytes,
}

impl ConnectionPacketSentEvent {
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
