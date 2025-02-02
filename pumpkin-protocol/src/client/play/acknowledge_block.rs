use pumpkin_data::packet::clientbound::PLAY_BLOCK_CHANGED_ACK;
use pumpkin_macros::client_packet;
use serde::Serialize;

use crate::VarInt;

#[derive(Serialize)]
#[client_packet(PLAY_BLOCK_CHANGED_ACK)]
pub struct CAcknowledgeBlockChange {
    sequence_id: VarInt,
}

impl CAcknowledgeBlockChange {
    pub fn new(sequence_id: VarInt) -> Self {
        Self { sequence_id }
    }
}
