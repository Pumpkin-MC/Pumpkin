use pumpkin_macros::packet;

use crate::{codec::var_int::VarInt, serial::PacketWrite};

#[derive(PacketWrite)]
#[packet(70)]
pub struct CChunkRadiusUpdate {
    // https://mojang.github.io/bedrock-protocol-docs/html/ChunkRadiusUpdatedPacket.html
    pub chunk_radius: VarInt,
}
