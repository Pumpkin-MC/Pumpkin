use pumpkin_data::packet::clientbound::PLAY_LEVEL_PARTICLES;
use pumpkin_macros::packet;
use pumpkin_util::math::vector3::Vector3;
use serde::Serialize;

use crate::{VarInt, ser::network_serialize_no_prefix};

#[derive(Serialize)]
#[packet(PLAY_LEVEL_PARTICLES)]
pub struct CParticle<'a> {
    force_spawn: bool,
    /// If true, particle distance increases from 256 to 65536.
    important: bool,
    position: Vector3<f64>,
    offset: Vector3<f32>,
    max_speed: f32,
    particle_count: i32,
    particle_id: VarInt,
    #[serde(serialize_with = "network_serialize_no_prefix")]
    data: &'a [u8],
}

impl<'a> CParticle<'a> {
    #[expect(clippy::too_many_arguments)]
    pub fn new(
        force_spawn: bool,
        important: bool,
        position: Vector3<f64>,
        offset: Vector3<f32>,
        max_speed: f32,
        particle_count: i32,
        particle_id: VarInt,
        data: &'a [u8],
    ) -> Self {
        Self {
            force_spawn,
            important,
            position,
            offset,
            max_speed,
            particle_count,
            particle_id,
            data,
        }
    }
}
