use pumpkin_macros::packet;

use crate::{codec::var_int::VarInt, serial::PacketRead};

#[derive(PacketRead, Debug)]
#[packet(69)]
pub struct SRequestChunkRadius {
    // https://mojang.github.io/bedrock-protocol-docs/html/RequestChunkRadiusPacket.html
    pub chunk_radius: VarInt,
    pub max_radius: u8,
}
