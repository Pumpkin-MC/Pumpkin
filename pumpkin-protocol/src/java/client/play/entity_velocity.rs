use pumpkin_data::packet::clientbound::PLAY_SET_ENTITY_MOTION;
use pumpkin_macros::packet;
use pumpkin_util::math::vector3::Vector3;
use serde::Serialize;

use crate::{VarInt, codec::velocity::Velocity};

#[derive(Serialize)]
#[packet(PLAY_SET_ENTITY_MOTION)]
pub struct CEntityVelocity {
    pub entity_id: VarInt,
    pub velocity: Velocity,
}

impl CEntityVelocity {
    pub fn new(entity_id: VarInt, velocity: Vector3<f64>) -> Self {
        Self {
            entity_id,
            velocity: Velocity(velocity),
        }
    }
}
