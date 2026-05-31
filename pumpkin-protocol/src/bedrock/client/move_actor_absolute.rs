use crate::{codec::var_ulong::VarULong, serial::PacketWrite};
use pumpkin_macros::packet;
use pumpkin_util::math::vector3::Vector3;

#[derive(PacketWrite)]
#[packet(18)]
pub struct CMoveActorAbsolute {
    pub entity_runtime_id: VarULong,
    pub flags: u8,
    pub position: Vector3<f32>,
    pub pitch: u8,
    pub yaw: u8,
    pub head_yaw: u8,
}

impl CMoveActorAbsolute {
    pub const FLAG_ON_GROUND: u8 = 0x01;
    pub const FLAG_TELEPORT: u8 = 0x02;
    pub const FLAG_FORCE_MOVE: u8 = 0x04;

    #[must_use]
    pub const fn new(
        entity_runtime_id: VarULong,
        flags: u8,
        position: Vector3<f32>,
        pitch: u8,
        yaw: u8,
        head_yaw: u8,
    ) -> Self {
        Self {
            entity_runtime_id,
            flags,
            position,
            pitch,
            yaw,
            head_yaw,
        }
    }
}
