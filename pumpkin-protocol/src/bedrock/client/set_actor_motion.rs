use pumpkin_macros::packet;
use pumpkin_util::math::vector3::Vector3;

use crate::{codec::var_ulong::VarULong, serial::PacketWrite};

#[derive(PacketWrite)]
#[packet(40)]
pub struct CSetActorMotion {
    pub target_runtime_id: VarULong,
    pub motion: Vector3<f32>,
    pub tick: VarULong,
}
