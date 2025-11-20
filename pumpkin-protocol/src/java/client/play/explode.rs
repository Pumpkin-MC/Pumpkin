use pumpkin_data::packet::clientbound::PLAY_EXPLODE;
use pumpkin_macros::packet;
use pumpkin_util::math::vector3::Vector3;
use serde::Serialize;

use crate::{IdOr, SoundEvent, codec::var_int::VarInt};

#[derive(Serialize)]
#[packet(PLAY_EXPLODE)]
pub struct CExplosion {
    pub center: Vector3<f64>,
    pub radius: f32,
    pub block_count: i32,
    pub knockback: Option<Vector3<f64>>,
    pub particle: VarInt,
    pub sound: IdOr<SoundEvent>,
    pub block_particles_pool_size: VarInt,
}

impl CExplosion {
    pub fn new(
        center: Vector3<f64>,
        radius: f32,
        block_count: i32,
        knockback: Option<Vector3<f64>>,
        particle: VarInt,
        sound: IdOr<SoundEvent>,
    ) -> Self {
        Self {
            center,
            radius,
            block_count,
            knockback,
            particle,
            sound,
            block_particles_pool_size: VarInt(0),
        }
    }
}
