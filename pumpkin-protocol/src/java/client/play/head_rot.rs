use pumpkin_data::packet::clientbound::PLAY_ROTATE_HEAD;
use pumpkin_macros::java_packet;
use serde::{Deserialize, Serialize};

use crate::VarInt;

/// Rotates an entity's head to a specific yaw.
///
/// In Minecraft, an entity's "body yaw" and "head yaw" are separate.
/// While standard movement packets update the body, this packet is
/// required to make an entity (like a player or a mob) look in a
/// specific direction without necessarily turning its entire body.
#[derive(Serialize, Deserialize)]
#[java_packet(PLAY_ROTATE_HEAD)]
pub struct CHeadRot {
    /// The Entity ID of the entity whose head is rotating.
    pub entity_id: VarInt,
    /// The new head yaw, in steps of 1/256 of a full turn (0-255).
    pub head_yaw: u8,
}

impl CHeadRot {
    #[must_use]
    pub fn new(entity_id: VarInt, head_yaw: u8) -> Self {
        Self {
            entity_id,
            head_yaw,
        }
    }
}
