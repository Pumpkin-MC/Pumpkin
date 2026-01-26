use pumpkin_data::packet::clientbound::PLAY_MOVE_ENTITY_ROT;
use pumpkin_macros::java_packet;
use serde::Serialize;

use crate::VarInt;

#[derive(Serialize)]
#[java_packet(PLAY_MOVE_ENTITY_ROT)]
pub struct CUpdateEntityRot {
    pub entity_id: VarInt,
    pub yaw: u8,
    pub pitch: u8,
    pub on_ground: bool,
}

impl CUpdateEntityRot {
    pub fn new(entity_id: VarInt, yaw: u8, pitch: u8, on_ground: bool) -> Self {
        Self {
            entity_id,
            yaw,
            pitch,
            on_ground,
        }
    }
}
