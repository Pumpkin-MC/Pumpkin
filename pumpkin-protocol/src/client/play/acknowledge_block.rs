use pumpkin_data::packet::clientbound::PLAY_BLOCK_CHANGED_ACK;
use pumpkin_macros::packet;
use serde::{Deserialize, Serialize};

use crate::VarInt;

#[derive(Serialize, Deserialize)]
#[packet(PLAY_BLOCK_CHANGED_ACK)]
pub struct CAcknowledgeBlockChange {
    pub sequence_id: VarInt,
}

impl CAcknowledgeBlockChange {
    pub fn new(sequence_id: VarInt) -> Self {
        Self { sequence_id }
    }
}
