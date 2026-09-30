// Last verified for v2169

use pumpkin_macros::packet;
use pumpkin_util::math::vector3::Vector3;

use crate::{codec::var_ulong::VarULong, serial::PacketRead};

#[derive(Debug, PacketRead)]
#[packet(33)]
pub struct SInteract {
    pub action: Action,
    pub target_runtime_id: VarULong,
    pub position: Option<Vector3<f32>>,
}

#[derive(Debug, PacketRead)]
#[repr(u8)]
pub enum Action {
    Invalid = 0,
    StopRiding = 3,
    InteractUpdate = 4,
    NpcOpen = 5,
    OpenInventory = 6,
}
