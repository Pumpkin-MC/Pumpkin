use pumpkin_data::packet::clientbound::PLAY_ENTITY_POSITION_SYNC;
use pumpkin_macros::packet;
use pumpkin_util::math::vector3::Vector3;
use serde::Serialize;

use crate::VarInt;

/// Synchronize entity position and rotation to the client.
/// The entity must not be the player itself, nor its vehicles.
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
