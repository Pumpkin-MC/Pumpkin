use pumpkin_macros::client_packet;
use pumpkin_util::math::vector3::Vector3;
use serde::Serialize;

use crate::VarInt;

#[derive(Serialize)]
#[client_packet("play:move_entity_pos")]
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
