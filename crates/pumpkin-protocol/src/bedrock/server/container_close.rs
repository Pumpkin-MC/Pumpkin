use pumpkin_macros::packet;

use crate::serial::{PacketRead, PacketWrite};

#[derive(Debug, PacketWrite, PacketRead)]
#[packet(47)]
pub struct SContainerClose {
    // https://mojang.github.io/bedrock-protocol-docs/html/ContainerClosePacket.html
    pub container_id: u8,
    pub container_type: u8,
    pub server_initiated: bool,
}
