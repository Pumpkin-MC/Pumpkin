// Last verified for v2169

use crate::{
    bedrock::network_item::NetworkItemStackDescriptor, codec::var_ulong::VarULong,
    serial::PacketRead,
};
use pumpkin_macros::packet;

#[derive(Debug, PacketRead)]
#[packet(31)]
pub struct SMobEquipment {
    pub entity_runtime_id: VarULong,
    pub item: NetworkItemStackDescriptor,
    pub slot: u8,
    pub selected_slot: u8,
    pub container_id: u8,
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use crate::serial::PacketWrite;

    use super::*;

    #[test]
    fn mob_equipment_reads_network_item_stack_descriptor() {
        let item = NetworkItemStackDescriptor::default();
        let mut encoded = Vec::new();
        VarULong(42).write(&mut encoded).unwrap();
        item.write(&mut encoded).unwrap();
        [3u8, 4, 5].write(&mut encoded).unwrap();

        let mut reader = Cursor::new(encoded);
        let packet = SMobEquipment::read(&mut reader).unwrap();

        assert_eq!(packet.entity_runtime_id.0, 42);
        assert_eq!(packet.item.id, 0);
        assert_eq!(packet.slot, 3);
        assert_eq!(packet.selected_slot, 4);
        assert_eq!(packet.container_id, 5);
        assert_eq!(reader.position(), reader.get_ref().len() as u64);
    }
}
