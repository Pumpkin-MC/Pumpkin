// Last verified for v2169

use crate::{
    bedrock::network_item::{FullContainerName, NetworkItemStackDescriptor},
    codec::var_uint::VarUInt,
    serial::PacketWrite,
};
use pumpkin_macros::packet;

#[derive(PacketWrite)]
#[packet(50)]
pub struct CInventorySlot {
    pub container_id: VarUInt,
    pub slot: VarUInt,
    pub full_container_name: Option<FullContainerName>,
    pub storage_item: Option<NetworkItemStackDescriptor>,
    pub item: NetworkItemStackDescriptor,
}
