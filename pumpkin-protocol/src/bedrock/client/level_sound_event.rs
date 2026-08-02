use pumpkin_macros::packet;
use pumpkin_util::math::vector3::Vector3;

use crate::{
    codec::{var_int::VarInt, var_uint::VarUInt},
    serial::PacketWrite,
};

/// Sent by the server to spawn a visual particle effect at a specific 3D location in the world.
///
/// Packet ID: `27`
/// Ref: <https://mojang.github.io/bedrock-protocol-docs/html/LevelSoundEventPacket.html>
#[derive(PacketWrite)]
#[packet(27)]
pub struct CLevelSoundEvent {
    pub sound_id: VarUInt,
    pub position: Vector3<f32>,
    pub extra_data: VarInt,
    pub entity_type: String,
    pub is_baby_mob: bool,
    pub is_global: bool,
}
