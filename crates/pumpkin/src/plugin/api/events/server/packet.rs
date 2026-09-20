use bytes::Bytes;
use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

use crate::entity::player::Player;
use pumpkin_protocol::ConnectionState;
use pumpkin_util::version::JavaMinecraftVersion;

#[cancellable]
#[derive(Event, Clone)]
pub struct PacketReceivedEvent {
    /// `None` in login and configuration state, where no `Player` exists yet.
    pub player: Option<Arc<Player>>,
    /// Same value before and after the login handover.
    pub connection_id: u64,
    pub version: JavaMinecraftVersion,
    pub state: ConnectionState,
    pub packet_id: i32,
    pub payload: Bytes,
    /// Written once the handler returns, even if cancelled; not fired back through [`PacketSentEvent`].
    pub reply_packets: Vec<(i32, Bytes)>,
}

impl PacketReceivedEvent {
    #[must_use]
    pub const fn new(
        player: Option<Arc<Player>>,
        connection_id: u64,
        version: JavaMinecraftVersion,
        state: ConnectionState,
        packet_id: i32,
        payload: Bytes,
    ) -> Self {
        Self {
            player,
            connection_id,
            version,
            state,
            packet_id,
            payload,
            reply_packets: Vec::new(),
            cancelled: false,
        }
    }
}

#[cancellable]
#[derive(Event, Clone)]
pub struct PacketSentEvent {
    /// `None` in login and configuration state, where no `Player` exists yet.
    pub player: Option<Arc<Player>>,
    /// Same value before and after the login handover.
    pub connection_id: u64,
    pub version: JavaMinecraftVersion,
    pub state: ConnectionState,
    pub packet_id: i32,
    pub payload: Bytes,
    /// Written right after this packet, in order, and not fired back here.
    pub extra_packets: Vec<(i32, Bytes)>,
    pub packet: Arc<dyn std::any::Any + Send + Sync>,
}

impl PacketSentEvent {
    #[must_use]
    pub fn new(
        player: Option<Arc<Player>>,
        connection_id: u64,
        version: JavaMinecraftVersion,
        state: ConnectionState,
        packet_id: i32,
        payload: Bytes,
        packet: Arc<dyn std::any::Any + Send + Sync>,
    ) -> Self {
        Self {
            player,
            connection_id,
            version,
            state,
            packet_id,
            payload,
            extra_packets: Vec::new(),
            packet,
            cancelled: false,
        }
    }
}
