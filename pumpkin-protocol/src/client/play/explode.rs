use pumpkin_data::packet::clientbound::PLAY_EXPLODE;
use pumpkin_macros::packet;
use pumpkin_util::math::vector3::Vector3;
use serde::Serialize;

use crate::{IDOrSoundEvent, codec::var_int::VarInt};

#[derive(Serialize)]
#[packet(PLAY_EXPLODE)]
pub struct CExplosion {
    center: Vector3<f64>,
    knockback: Option<Vector3<f64>>,
    particle: VarInt,
    sound: IDOrSoundEvent,
}

impl CExplosion {
    pub fn new(
        center: Vector3<f64>,
        knockback: Option<Vector3<f64>>,
        particle: VarInt,
        sound: IDOrSoundEvent,
    ) -> Self {
        Self {
            center,
            knockback,
            particle,
            sound,
        }
    }
}
