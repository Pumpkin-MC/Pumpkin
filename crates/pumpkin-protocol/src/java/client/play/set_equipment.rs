use std::io::Write;

use crate::{
    ClientPacket, ServerPacket, VarInt,
    codec::item_stack_seralizer::ItemStackSerializer,
    ser::{NetworkReadExt, NetworkWriteExt, ReadingError, WritingError},
};
use pumpkin_data::packet::clientbound::play::SET_EQUIPMENT;
use pumpkin_macros::java_packet;

#[java_packet(SET_EQUIPMENT)]
#[derive(Clone)]
pub struct CSetEquipment {
    pub entity_id: VarInt,
    pub equipment: Vec<(i8, ItemStackSerializer<'static>)>,
}

impl CSetEquipment {
    #[must_use]
    pub const fn new(
        entity_id: VarInt,
        equipment: Vec<(i8, ItemStackSerializer<'static>)>,
    ) -> Self {
        Self {
            entity_id,
            equipment,
        }
    }
}

impl ClientPacket for CSetEquipment {
    fn write_packet_data(&self, mut write: impl Write) -> Result<(), WritingError> {
        write.write_var_int(&self.entity_id)?;

        {
            let size = self.equipment.len();
            for (i, equipment) in self.equipment.iter().enumerate() {
                let slot = equipment.0;
                let last = i == size - 1;
                let slot_byte = if last { slot } else { slot | -128 };
                write.write_i8(slot_byte)?;
                equipment.1.write(&mut write)?;
            }
        }

        Ok(())
    }
}

impl<'a> ServerPacket<'a> for CSetEquipment {
    fn read(bytebuf: &mut &'a [u8]) -> Result<Self, ReadingError> {
        let entity_id = bytebuf.get_var_int()?;

        let equipment = {
            let mut equipment = Vec::new();
            loop {
                let value = bytebuf.get_u8()?;
                let slot = (value & 0x7F) as i8;
                let item = ItemStackSerializer::read(bytebuf)?;
                equipment.push((slot, item));
                if (value & 0x80) == 0 {
                    break;
                }
            }
            equipment
        };

        Ok(Self {
            entity_id,
            equipment,
        })
    }
}

#[cfg(test)]
mod tests {
    use pumpkin_data::item::Item;
    use pumpkin_data::item_stack::ItemStack;
    use pumpkin_data::packet::clientbound::play::SET_EQUIPMENT;
    use pumpkin_util::version::JavaMinecraftVersion;

    use crate::{
        ClientPacket, VarInt, codec::item_stack_seralizer::ItemStackSerializer, packet::JavaPacket,
        ser::NetworkReadExt,
    };

    use super::CSetEquipment;

    fn encoded_armor(_version: JavaMinecraftVersion) -> Vec<u8> {
        let chest = ItemStackSerializer::from(ItemStack::new(1, &Item::DIAMOND_CHESTPLATE));
        let legs = ItemStackSerializer::from(ItemStack::new(1, &Item::DIAMOND_LEGGINGS));
        let packet = CSetEquipment::new(VarInt(42), vec![(4, chest), (3, legs)]);
        let mut buf = Vec::new();
        packet.write_packet_data(&mut buf).unwrap();
        buf
    }

    fn assert_armor_payload(bytes: &[u8]) {
        let mut cursor = bytes;
        let entity_id = cursor.get_var_int().unwrap();
        assert_eq!(entity_id, VarInt(42));

        let first_slot = cursor.get_i8().unwrap();
        assert_eq!(
            first_slot,
            4i8 | -128,
            "chest must set the continuation bit"
        );
        let first_item = ItemStackSerializer::read(&mut cursor).unwrap();
        assert_eq!(first_item.0.item.id, Item::DIAMOND_CHESTPLATE.id);
        assert_eq!(first_item.0.item_count, 1);

        let second_slot = cursor.get_i8().unwrap();
        assert_eq!(
            second_slot, 3,
            "legs is the last entry and must not set 0x80"
        );
        let second_item = ItemStackSerializer::read(&mut cursor).unwrap();
        assert_eq!(second_item.0.item.id, Item::DIAMOND_LEGGINGS.id);
        assert!(cursor.is_empty());
    }

    #[test]
    fn set_equipment_packet_id_for_26_3() {
        assert_eq!(CSetEquipment::PACKET_ID, SET_EQUIPMENT.to_id());
        assert_eq!(CSetEquipment::PACKET_ID, 104);
    }

    #[test]
    fn armour_slots_encode() {
        let version = pumpkin_data::packet::CURRENT_MC_VERSION;
        assert_armor_payload(&encoded_armor(version));
    }
}
