use std::io::Write;

use crate::{
    ClientPacket, ServerPacket, VarInt,
    codec::item_stack_seralizer::ItemStackSerializer,
    ser::{NetworkReadExt, NetworkWriteExt, ReadingError, WritingError},
};
use pumpkin_data::packet::clientbound::play::SET_EQUIPMENT;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

#[must_use]
pub fn slot_to_version(slot: i8, version: &JavaMinecraftVersion) -> i8 {
    if *version <= JavaMinecraftVersion::V_1_8 {
        match slot {
            0 => 0, // MainHand -> Held
            2 => 1, // Feet -> Boots
            3 => 2, // Legs -> Leggings
            4 => 3, // Chest -> Chestplate
            5 => 4, // Head -> Helmet
            _ => slot,
        }
    } else {
        slot
    }
}

#[must_use]
pub fn slot_from_version(slot: i8, version: &JavaMinecraftVersion) -> i8 {
    if *version <= JavaMinecraftVersion::V_1_8 {
        match slot {
            0 => 0, // Held -> MainHand
            1 => 2, // Boots -> Feet
            2 => 3, // Leggings -> Legs
            3 => 4, // Chestplate -> Chest
            4 => 5, // Helmet -> Head
            _ => slot,
        }
    } else {
        slot
    }
}

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
    fn write_packet_data(
        &self,
        mut write: impl Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        if *version <= JavaMinecraftVersion::V_1_7_6 {
            write.write_i32_be(self.entity_id.0)?;
        } else {
            write.write_var_int(&self.entity_id)?;
        }

        if *version >= JavaMinecraftVersion::V_1_16 {
            let size = self.equipment.len();
            for (i, equipment) in self.equipment.iter().enumerate() {
                let slot = equipment.0;
                let last = i == size - 1;
                let slot_byte = if last { slot } else { slot | -128 };
                write.write_i8(slot_byte)?;
                equipment.1.write_with_version(&mut write, version)?;
            }
        } else if let Some(equipment) = self.equipment.first() {
            let slot = slot_to_version(equipment.0, version);
            if *version >= JavaMinecraftVersion::V_1_9 {
                write.write_var_int(&VarInt(i32::from(slot)))?;
            } else {
                write.write_i16_be(i16::from(slot))?;
            }
            equipment.1.write_with_version(&mut write, version)?;
        }

        Ok(())
    }
}

impl<'a> ServerPacket<'a> for CSetEquipment {
    fn read(bytebuf: &mut &'a [u8], version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        let entity_id = if *version <= JavaMinecraftVersion::V_1_7_6 {
            VarInt(bytebuf.get_i32_be()?)
        } else {
            bytebuf.get_var_int()?
        };

        let equipment = if *version >= JavaMinecraftVersion::V_1_16 {
            let mut equipment = Vec::new();
            loop {
                let value = bytebuf.get_u8()?;
                let slot = (value & 0x7F) as i8;
                let item = ItemStackSerializer::read_with_version(bytebuf, version)?;
                equipment.push((slot, item));
                if (value & 0x80) == 0 {
                    break;
                }
            }
            equipment
        } else if *version >= JavaMinecraftVersion::V_1_9 {
            let slot = bytebuf.get_var_int()?.0 as i8;
            let item = ItemStackSerializer::read_with_version(bytebuf, version)?;
            vec![(slot, item)]
        } else {
            let raw_slot = bytebuf.get_i16_be()? as i8;
            let slot = slot_from_version(raw_slot, version);
            let item = ItemStackSerializer::read_with_version(bytebuf, version)?;
            vec![(slot, item)]
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
        ClientPacket, VarInt, codec::item_stack_seralizer::ItemStackSerializer,
        packet::MultiVersionJavaPacket, ser::NetworkReadExt,
    };

    use super::CSetEquipment;

    fn encoded_armor(version: JavaMinecraftVersion) -> Vec<u8> {
        let chest = ItemStackSerializer::from(ItemStack::new(1, &Item::DIAMOND_CHESTPLATE));
        let legs = ItemStackSerializer::from(ItemStack::new(1, &Item::DIAMOND_LEGGINGS));
        let packet = CSetEquipment::new(VarInt(42), vec![(4, chest), (3, legs)]);
        let mut buf = Vec::new();
        packet.write_packet_data(&mut buf, &version).unwrap();
        buf
    }

    fn assert_armor_payload(bytes: &[u8], version: JavaMinecraftVersion) {
        let mut cursor = bytes;
        let entity_id = cursor.get_var_int().unwrap();
        assert_eq!(entity_id, VarInt(42));

        let first_slot = cursor.get_i8().unwrap();
        assert_eq!(
            first_slot,
            4i8 | -128,
            "chest must set the continuation bit"
        );
        let first_item = ItemStackSerializer::read_with_version(&mut cursor, &version).unwrap();
        assert_eq!(first_item.0.item.id, Item::DIAMOND_CHESTPLATE.id);
        assert_eq!(first_item.0.item_count, 1);

        let second_slot = cursor.get_i8().unwrap();
        assert_eq!(
            second_slot, 3,
            "legs is the last entry and must not set 0x80"
        );
        let second_item = ItemStackSerializer::read_with_version(&mut cursor, &version).unwrap();
        assert_eq!(second_item.0.item.id, Item::DIAMOND_LEGGINGS.id);
        assert!(cursor.is_empty());
    }

    #[test]
    fn set_equipment_packet_id_for_1_21_and_26() {
        assert_eq!(
            CSetEquipment::to_id(JavaMinecraftVersion::V_1_21),
            SET_EQUIPMENT.to_id(JavaMinecraftVersion::V_1_21)
        );
        assert_eq!(CSetEquipment::to_id(JavaMinecraftVersion::V_1_21), 91);
        assert_eq!(CSetEquipment::to_id(JavaMinecraftVersion::V_26_2), 102);
    }

    #[test]
    fn armour_slots_encode_for_1_21() {
        let version = JavaMinecraftVersion::V_1_21;
        assert_armor_payload(&encoded_armor(version), version);
    }

    #[test]
    fn armour_slots_encode_for_26_2() {
        let version = JavaMinecraftVersion::V_26_2;
        assert_armor_payload(&encoded_armor(version), version);
    }
}
