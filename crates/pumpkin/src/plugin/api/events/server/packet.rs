use bytes::Bytes;
use pumpkin_macros::{Event, cancellable};
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
            cancelled: false,
        }
    }
}
