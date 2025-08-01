use pumpkin_macros::packet;

use crate::{
    bedrock::network_item::NetworkItemStackDescriptor, codec::var_uint::VarUInt,
    serial::PacketWrite,
};

#[derive(PacketWrite)]
#[packet(49)]
pub struct CInventoryContent {
    // https://mojang.github.io/bedrock-protocol-docs/html/InventoryContentPacket.html
    pub inventory_id: VarUInt,
    pub slots: Vec<NetworkItemStackDescriptor>,
    pub container_name: u8,
    pub dynamic_id: Option<u32>,
    /// Use NetworkItemDescriptor if none
    pub storage_item: NetworkItemStackDescriptor,
}
