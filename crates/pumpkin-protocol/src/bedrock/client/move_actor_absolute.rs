// Last verified for v2169

use crate::{codec::var_ulong::VarULong, serial::PacketWrite};
use pumpkin_macros::packet;
use pumpkin_util::math::vector3::Vector3;

#[derive(PacketWrite)]
#[packet(18)]
pub struct CMoveActorAbsolute {
    pub actor_runtime_id: VarULong,
    pub header: u8,
    pub position: Vector3<f32>,
    pub rotation_x: u8,
    pub rotation_y: u8,
    pub rotation_y_head: u8,
}

impl CMoveActorAbsolute {
    pub const FLAG_ON_GROUND: u8 = 0x01;
    pub const FLAG_TELEPORT: u8 = 0x02;
    pub const FLAG_FORCE_MOVE: u8 = 0x04;
}
