use pumpkin_macros::packet;
use uuid::Uuid;

use crate::{
    codec::{var_int::VarInt, var_uint::VarUInt},
    serial::PacketRead,
};

#[derive(Debug, PacketRead)]
#[packet(77)]
pub struct SCommandRequest {
    // https://mojang.github.io/bedrock-protocol-docs/html/CommandRequestPacket.html
    pub command: String,
    pub command_type: VarUInt,
    pub command_uuid: Uuid,
    pub request_id: String,
    pub is_internal_source: bool,
    pub version: VarInt,
}
