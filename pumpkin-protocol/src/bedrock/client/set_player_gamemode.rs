use crate::{codec::var_int::VarInt, serial::PacketWrite};
use pumpkin_macros::packet;

#[derive(PacketWrite)]
#[packet(62)]
pub struct CSetPlayerGamemode {
    // https://mojang.github.io/bedrock-protocol-docs/html/SetPlayerGameTypePacket.html
    pub gamemode: VarInt,
}
