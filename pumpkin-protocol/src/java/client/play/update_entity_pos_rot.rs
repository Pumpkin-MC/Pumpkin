use pumpkin_data::packet::clientbound::PLAY_MOVE_ENTITY_POS_ROT;
use pumpkin_macros::java_packet;
use pumpkin_util::math::vector3::Vector3;
use serde::Serialize;

use crate::VarInt;

#[derive(Serialize)]
#[java_packet(PLAY_MOVE_ENTITY_POS_ROT)]
pub struct CUpdateEntityPosRot {
    pub entity_id: VarInt,
    pub delta: Vector3<i16>,
    pub yaw: u8,
    pub pitch: u8,
    pub on_ground: bool,
}

impl CUpdateEntityPosRot {
    #[must_use]
    pub fn new(
        entity_id: VarInt,
        delta: Vector3<i16>,
        yaw: u8,
        pitch: u8,
        on_ground: bool,
    ) -> Self {
        Self {
            entity_id,
            delta,
            yaw,
            pitch,
            on_ground,
        }
    }
}
