use pumpkin_data::packet::clientbound::PLAY_EXPLODE;
use pumpkin_macros::packet;
use pumpkin_util::math::vector3::Vector3;
use serde::{Deserialize, Serialize};

use crate::{IdOr, SoundEvent, codec::var_int::VarInt};

#[derive(Serialize, Deserialize)]
#[serde(bound(deserialize = "'a: 'de"))]
#[packet(PLAY_EXPLODE)]
pub struct CExplosion<'a> {
    center: Vector3<f64>,
    knockback: Option<Vector3<f64>>,
    particle: VarInt,
    #[serde(borrow)]
    sound: IdOr<SoundEvent<'a>>,
}

impl<'a> CExplosion<'a> {
    pub fn new(
        center: Vector3<f64>,
        knockback: Option<Vector3<f64>>,
        particle: VarInt,
        sound: IdOr<SoundEvent<'a>>,
    ) -> Self {
        Self {
            center,
            knockback,
            particle,
            sound,
        }
    }
}
