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

impl CSetActorMotion {
    #[must_use]
    pub const fn new(target_runtime_id: VarULong, motion: Vector3<f32>, tick: VarULong) -> Self {
        Self {
            target_runtime_id,
            motion,
            tick,
        }
    }
}
