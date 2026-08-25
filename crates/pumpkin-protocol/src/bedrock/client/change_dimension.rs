// Last verified for v2169

use pumpkin_macros::packet;
use pumpkin_util::math::vector3::Vector3;

use crate::{codec::var_int::VarInt, serial::PacketWrite};

#[derive(PacketWrite)]
#[packet(61)]
pub struct CChangeDimension {
    pub dimension_id: VarInt,
    pub position: Vector3<f32>,
    pub respawn: bool,
    pub loading_screen_id: Option<u32>,
}
