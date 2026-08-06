use pumpkin_macros::packet;

use crate::{
    bedrock::network_item::{FullContainerName, NetworkItemStackDescriptor},
    codec::var_uint::VarUInt,
    serial::PacketWrite,
};

#[derive(PacketWrite)]
#[packet(49)]
pub struct CInventoryContent {
    // https://mojang.github.io/bedrock-protocol-docs/docs/InventoryContentPacket.html
    pub container_id: VarUInt,
    pub slots: Vec<NetworkItemStackDescriptor>,
    pub full_container_name: FullContainerName,
    pub storage_item: NetworkItemStackDescriptor,
}
