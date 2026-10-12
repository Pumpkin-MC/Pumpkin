// Last verified for v2169

use crate::{
    bedrock::network_item::NetworkItemStackDescriptor, codec::var_ulong::VarULong,
    serial::PacketWrite,
};
use pumpkin_macros::packet;

/// Sent by the server to show the item equipped or held by a mob or other player.
#[derive(PacketWrite, Debug)]
#[packet(31)]
pub struct CMobEquipment {
    /// Runtime entity ID of the equipped actor.
    pub target_runtime_id: VarULong,
    /// Item descriptor for the equipped item stack.
    pub item: NetworkItemStackDescriptor,
    /// Slot index within the container.
    pub slot: u8,
    /// Currently selected hotbar slot.
    pub selected_slot: u8,
    /// Container identifier (e.g. inventory or armor).
    pub container_id: u8,
}
