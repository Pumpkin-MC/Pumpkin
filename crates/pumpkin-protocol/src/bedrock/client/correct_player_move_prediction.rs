// Last verified for v2169

use pumpkin_macros::packet;
use pumpkin_util::math::{vector2::Vector2, vector3::Vector3};

use crate::{codec::var_ulong::VarULong, serial::PacketWrite};

#[derive(PacketWrite)]
#[packet(161)]
pub struct CCorrectPlayerMovePrediction {
    pub prediction_type: u8,
    pub pos: Vector3<f32>,
    pub pos_delta: Vector3<f32>,
    pub rotation: Vector2<f32>,
    pub vehicle_angular_velocity: Option<f32>,
    pub on_ground: bool,
    pub tick: VarULong,
}
