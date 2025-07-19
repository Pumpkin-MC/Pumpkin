use pumpkin_macros::packet;
use pumpkin_util::math::vector3::Vector3;

use crate::{codec::var_ulong::VarULong, serial::PacketWrite};

#[derive(Debug, PacketWrite)]
#[packet(19)]
pub struct CMovePlayer {
    // https://mojang.github.io/bedrock-protocol-docs/html/MovePlayerPacket.html
    pub player_runtime_id: VarULong,
    pub position: Vector3<f32>,
    pub pitch: f32,
    pub yaw: f32,
    pub y_head_rotation: f32,
    pub position_mode: u8,
    pub on_ground: bool,
    pub riding_runtime_id: VarULong,
    pub tick: VarULong,
}
