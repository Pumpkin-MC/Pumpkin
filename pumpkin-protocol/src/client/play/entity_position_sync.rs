use pumpkin_data::packet::clientbound::PLAY_ENTITY_POSITION_SYNC;
use pumpkin_macros::packet;
use pumpkin_util::math::vector3::Vector3;
use serde::Serialize;

use crate::VarInt;

// https://minecraft.wiki/w/Java_Edition_protocol#Synchronize_Vehicle_Position
// Badly documented and confusing packet imo
#[packet(PLAY_ENTITY_POSITION_SYNC)]
#[derive(Serialize)]
pub struct CEntityPositionSync {
    entity_id: VarInt,
    position: Vector3<f64>,
    delta: Vector3<f64>,
    yaw: f32,
    pitch: f32,
    on_ground: bool,
}

impl CEntityPositionSync {
    pub fn new(
        entity_id: VarInt,
        position: Vector3<f64>,
        delta: Vector3<f64>,
        yaw: f32,
        pitch: f32,
        on_ground: bool,
    ) -> Self {
        Self {
            entity_id,
            position,
            delta,
            yaw,
            pitch,
            on_ground,
        }
    }
}
