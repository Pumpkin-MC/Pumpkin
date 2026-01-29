use pumpkin_data::packet::clientbound::PLAY_MOVE_ENTITY_POS;
use pumpkin_macros::java_packet;
use pumpkin_util::math::vector3::Vector3;
use serde::Serialize;

use crate::VarInt;

#[derive(Serialize)]
#[java_packet(PLAY_MOVE_ENTITY_POS)]
pub struct CUpdateEntityPos {
    pub entity_id: VarInt,
    pub delta: Vector3<i16>,
    pub on_ground: bool,
}

impl CUpdateEntityPos {
    #[must_use]
    pub const fn new(entity_id: VarInt, delta: Vector3<i16>, on_ground: bool) -> Self {
        Self {
            entity_id,
            delta,
            on_ground,
        }
    }
}
