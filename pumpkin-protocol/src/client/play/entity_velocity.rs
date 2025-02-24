use pumpkin_data::packet::clientbound::PLAY_SET_ENTITY_MOTION;
use pumpkin_macros::packet;
use pumpkin_util::math::vector3::Vector3;
use serde::Serialize;

use crate::VarInt;

#[derive(Serialize)]
#[packet(PLAY_SET_ENTITY_MOTION)]
pub struct CEntityVelocity {
    entity_id: VarInt,
    velocity: Vector3<i16>,
}

impl CEntityVelocity {
    pub fn new(entity_id: VarInt, velocity_x: f64, velocity_y: f64, velocity_z: f64) -> Self {
        Self {
            entity_id,
            velocity: Vector3::new(
                (velocity_x.clamp(-3.9, 3.9) * 8000.0) as i16,
                (velocity_y.clamp(-3.9, 3.9) * 8000.0) as i16,
                (velocity_z.clamp(-3.9, 3.9) * 8000.0) as i16,
            ),
        }
    }
}
