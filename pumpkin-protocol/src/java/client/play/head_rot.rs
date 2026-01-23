use pumpkin_data::packet::clientbound::PLAY_ROTATE_HEAD;
use pumpkin_macros::packet;
use serde::{Deserialize, Serialize};

use crate::VarInt;

#[derive(Serialize, Deserialize)]
#[packet(PLAY_ROTATE_HEAD)]
pub struct CHeadRot {
    pub entity_id: VarInt,
    pub head_yaw: u8,
}

impl CHeadRot {
    #[must_use]
    pub const fn new(entity_id: VarInt, head_yaw: u8) -> Self {
        Self {
            entity_id,
            head_yaw,
        }
    }
}
