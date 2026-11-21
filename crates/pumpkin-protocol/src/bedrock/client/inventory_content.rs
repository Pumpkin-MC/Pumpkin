// Last verified for v2169

use pumpkin_macros::packet;

use crate::{
    bedrock::network_item::{FullContainerName, NetworkItemStackDescriptor},
    codec::var_uint::VarUInt,
    serial::PacketWrite,
};

/// Sent by the server to synchronize the entire contents of an inventory or container window.
#[derive(PacketWrite)]
#[packet(49)]
pub struct CInventoryContent {
    /// Window ID of the container being populated.
    pub container_id: VarUInt,
    /// Array of item stacks filling every slot of the container.
    pub slots: Vec<NetworkItemStackDescriptor>,
    /// Full container name descriptor with dynamic session ID.
    pub full_container_name: FullContainerName,
    /// Storage item descriptor (e.g. shulker box item if inspecting inside a shulker).
    pub storage_item: NetworkItemStackDescriptor,
}
