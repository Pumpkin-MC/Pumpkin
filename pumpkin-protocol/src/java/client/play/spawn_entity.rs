use pumpkin_data::packet::clientbound::PLAY_ADD_ENTITY;
use pumpkin_macros::packet;
use pumpkin_util::math::vector3::Vector3;
use serde::Serialize;

use crate::{VarInt, codec::velocity::Velocity};

#[derive(Serialize)]
#[packet(PLAY_ADD_ENTITY)]
pub struct CSpawnEntity {
    pub entity_id: VarInt,
    #[serde(with = "uuid::serde::compact")]
    pub entity_uuid: uuid::Uuid,
    pub r#type: VarInt,
    pub position: Vector3<f64>,
    pub velocity: Velocity,
    pub pitch: u8,    // angle
    pub yaw: u8,      // angle
    pub head_yaw: u8, // angle
    pub data: VarInt,
}

impl CSpawnEntity {
    #[expect(clippy::too_many_arguments)]
    pub fn new(
        entity_id: VarInt,
        entity_uuid: uuid::Uuid,
        r#type: VarInt,
        position: Vector3<f64>,
        pitch: f32,    // angle
        yaw: f32,      // angle
        head_yaw: f32, // angle
        data: VarInt,
        velocity: Vector3<f64>,
    ) -> Self {
        Self {
            entity_id,
            entity_uuid,
            r#type,
            position,
            pitch: (pitch * 256.0 / 360.0).floor() as u8,
            yaw: (yaw.rem_euclid(360.0) * 256.0 / 360.0).floor() as u8,
            head_yaw: (head_yaw.rem_euclid(360.0) * 256.0 / 360.0).floor() as u8,
            data,
            velocity: Velocity(velocity),
        }
    }
}
