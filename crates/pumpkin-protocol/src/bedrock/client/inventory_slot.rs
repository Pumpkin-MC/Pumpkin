// Last verified for v2169

use crate::{
    bedrock::network_item::{FullContainerName, NetworkItemStackDescriptor},
    codec::var_uint::VarUInt,
    serial::PacketWrite,
};
use pumpkin_macros::packet;

/// Sent by the server to update a single item slot within an open inventory container.
#[derive(PacketWrite)]
#[packet(50)]
pub struct CInventorySlot {
    /// Window ID of the container being updated.
    pub container_id: VarUInt,
    /// Slot index within the container.
    pub slot: VarUInt,
    /// Optional full container descriptor.
    pub full_container_name: Option<FullContainerName>,
    /// Optional storage item descriptor.
    pub storage_item: Option<NetworkItemStackDescriptor>,
    /// New item stack descriptor placed into the slot.
    pub item: NetworkItemStackDescriptor,
}
