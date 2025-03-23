use pumpkin_data::packet::clientbound::PLAY_MOVE_ENTITY_POS;
use pumpkin_macros::packet;
use pumpkin_util::math::vector3::Vector3;
use serde::Serialize;

use crate::VarInt;

#[derive(Serialize)]
#[packet(PLAY_MOVE_ENTITY_POS)]
pub struct CUpdateEntityPos {
    entity_id: VarInt,
    delta: Vector3<i16>,
    on_ground: bool,
}

impl CUpdateEntityPos {
    pub fn new(entity_id: VarInt, delta: Vector3<i16>, on_ground: bool) -> Self {
        Self {
            entity_id,
            delta,
            on_ground,
        }
    }
}
