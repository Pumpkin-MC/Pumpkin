use crate::codec::var_int::VarInt;
use pumpkin_macros::packet;
use serde::Serialize;

#[derive(Serialize)]
#[packet(62)]
pub struct CSetPlayerGamemode {
    // https://mojang.github.io/bedrock-protocol-docs/html/SetPlayerGameTypePacket.html
    pub gamemode: VarInt,
}
