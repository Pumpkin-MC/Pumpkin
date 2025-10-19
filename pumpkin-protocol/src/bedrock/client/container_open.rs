use pumpkin_macros::packet;

use crate::{
    codec::{bedrock_block_pos::NetworkPos, var_long::VarLong},
    serial::PacketWrite,
};

#[derive(PacketWrite)]
#[packet(46)]
pub struct CContainerOpen {
    // https://mojang.github.io/bedrock-protocol-docs/html/ContainerOpenPacket.html
    pub container_id: u8,
    pub container_type: u8,
    pub position: NetworkPos,
    pub target_entity_id: VarLong,
}
