use pumpkin_macros::packet;
use pumpkin_util::math::vector3::Vector3;

use crate::{bedrock::respawn::RespawnState, codec::var_ulong::VarULong, serial::PacketRead};

#[derive(PacketRead)]
#[packet(45)]
pub struct SRespawn {
    pub position: Vector3<f32>,
    pub state: RespawnState,
    pub player_runtime_id: VarULong,
}
