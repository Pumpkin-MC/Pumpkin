use pumpkin_data::packet::clientbound::PLAY_BLOCK_CHANGED_ACK;
use pumpkin_macros::java_packet;
use serde::{Deserialize, Serialize};

use crate::VarInt;

/// Sent by the server to acknowledge a sequence of block changes initiated by the client.
///
/// This packet is critical for preventing "ghost blocks" and synchronization issues.
/// It tells the client that the server has processed all actions up to a specific point.
#[derive(Serialize, Deserialize)]
#[java_packet(PLAY_BLOCK_CHANGED_ACK)]
pub struct CAcknowledgeBlockChange {
    /// The ID of the last sequence processed by the server.
    ///
    /// The client increments this ID every time it starts a sequence of actions
    /// (like breaking or placing a block), and the server must mirror it back
    /// to confirm processing is complete.
    pub sequence_id: VarInt,
}

impl CAcknowledgeBlockChange {
    pub fn new(sequence_id: VarInt) -> Self {
        Self { sequence_id }
    }
}
