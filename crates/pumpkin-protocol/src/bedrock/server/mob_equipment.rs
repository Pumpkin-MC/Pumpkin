use crate::{
    bedrock::network_item::NetworkItemStackDescriptor, codec::var_ulong::VarULong,
    serial::PacketRead,
};
use pumpkin_macros::packet;
use std::io::{Error, Read};

#[derive(Debug)]
#[packet(31)]
pub struct SMobEquipment {
    pub entity_runtime_id: VarULong,
    pub item: NetworkItemStackDescriptor,
    pub inventory_slot: u8,
    pub hotbar_slot: u8,
    pub window_id: u8,
}

impl PacketRead for SMobEquipment {
    fn read<R: Read>(reader: &mut R) -> Result<Self, Error> {
        let entity_runtime_id = VarULong::read(reader)?;
        let item = NetworkItemStackDescriptor::read(reader)?;
        let inventory_slot = u8::read(reader)?;
        let hotbar_slot = u8::read(reader)?;
        let window_id = u8::read(reader)?;

        Ok(Self {
            entity_runtime_id,
            item,
            inventory_slot,
            hotbar_slot,
            window_id,
        })
    }
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
        assert_eq!(packet.inventory_slot, 3);
        assert_eq!(packet.hotbar_slot, 4);
        assert_eq!(packet.window_id, 5);
        assert_eq!(reader.position(), reader.get_ref().len() as u64);
    }
}
