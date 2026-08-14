use crate::{
    bedrock::network_item::NetworkItemStackDescriptor, codec::var_ulong::VarULong,
    serial::PacketWrite,
};
use pumpkin_macros::packet;

#[derive(PacketWrite, Debug)]
#[packet(31)]
pub struct CMobEquipment {
    pub entity_runtime_id: VarULong,
    pub item: NetworkItemStackDescriptor,
    pub inventory_slot: u8,
    pub hotbar_slot: u8,
    pub window_id: u8,
}

impl CMobEquipment {
    #[must_use]
    pub const fn new(
        entity_runtime_id: u64,
        item: NetworkItemStackDescriptor,
        inventory_slot: u8,
        hotbar_slot: u8,
        window_id: u8,
    ) -> Self {
        Self {
            entity_runtime_id: VarULong(entity_runtime_id),
            item,
            inventory_slot,
            hotbar_slot,
            window_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::packet::Packet;

    #[test]
    fn mob_equipment_packet_id() {
        assert_eq!(CMobEquipment::PACKET_ID, 31);
    }
}
