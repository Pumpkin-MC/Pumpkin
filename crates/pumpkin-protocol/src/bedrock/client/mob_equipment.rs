// Last verified for v2169

use crate::{
    bedrock::network_item::NetworkItemStackDescriptor, codec::var_ulong::VarULong,
    serial::PacketWrite,
};
use pumpkin_macros::packet;

#[derive(PacketWrite, Debug)]
#[packet(31)]
pub struct CMobEquipment {
    pub target_runtime_id: VarULong,
    pub item: NetworkItemStackDescriptor,
    pub slot: u8,
    pub selected_slot: u8,
    pub container_id: u8,
}
