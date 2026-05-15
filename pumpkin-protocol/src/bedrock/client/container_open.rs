use pumpkin_macros::packet;
use pumpkin_util::math::position::BlockPos;

use crate::{codec::var_long::VarLong, serial::PacketWrite};

#[derive(PacketWrite)]
#[packet(46)]
pub struct CContainerOpen {
    // https://mojang.github.io/bedrock-protocol-docs/html/ContainerOpenPacket.html
    pub container_id: u8,
    pub container_type: u8,
    pub position: BlockPos,
    pub target_entity_id: VarLong,
}
