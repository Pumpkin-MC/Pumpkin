use pumpkin_macros::packet;
use pumpkin_util::math::vector3::Vector3;

use crate::{codec::var_ulong::VarULong, serial::PacketWrite};

#[derive(PacketWrite)]
#[packet(161)]
pub struct CCorrectPlayerMove {
    // https://mojang.github.io/bedrock-protocol-docs/html/CorrectPlayerMovePredictionPacket.html
    pub prediction_type: u8,
    pub pos: Vector3<f32>,
    pub pos_delta: Vector3<f32>,
    pub on_ground: bool,
    pub tick: VarULong,
}
