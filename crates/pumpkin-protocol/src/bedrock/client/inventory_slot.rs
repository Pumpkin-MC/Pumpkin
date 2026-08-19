use crate::{
    bedrock::network_item::{FullContainerName, NetworkItemStackDescriptor},
    codec::var_uint::VarUInt,
    serial::PacketWrite,
};
use pumpkin_macros::packet;

#[derive(PacketWrite)]
#[packet(50)]
pub struct CInventorySlot {
    pub window_id: VarUInt,
    pub inventory_slot: VarUInt,
    pub container_name: Option<FullContainerName>,
    pub storage: Option<NetworkItemStackDescriptor>,
    pub item: NetworkItemStackDescriptor,
}
