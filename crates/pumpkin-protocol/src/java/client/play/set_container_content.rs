use std::io::Write;

use crate::VarInt;
use crate::codec::item_stack_seralizer::ItemStackSerializer;
use crate::ser::{NetworkReadExt, ReadingError};
use crate::{ClientPacket, ServerPacket, WritingError, ser::NetworkWriteExt};

use pumpkin_data::packet::clientbound::play::CONTAINER_SET_CONTENT;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(CONTAINER_SET_CONTENT)]
pub struct CSetContainerContent<'a> {
    pub window_id: VarInt,
    pub state_id: VarInt,
    pub slot_data: &'a [ItemStackSerializer<'a>],
    pub carried_item: &'a ItemStackSerializer<'a>,
}

impl<'a> CSetContainerContent<'a> {
    #[must_use]
    pub const fn new(
        window_id: VarInt,
        state_id: VarInt,
        slots: &'a [ItemStackSerializer],
        carried_item: &'a ItemStackSerializer,
    ) -> Self {
        Self {
            window_id,
            state_id,
            slot_data: slots,
            carried_item,
        }
    }
}

impl ClientPacket for CSetContainerContent<'_> {
    fn write_packet_data(
        &self,
        write: impl Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        let mut write = write;

        write.write_container_id(&self.window_id, version)?;
        if *version >= JavaMinecraftVersion::V_1_17_1 {
            write.write_var_int(&self.state_id)?;
        }

        // Window 0 contains the 1.9+ offhand slot at index 45, but 1.7 and 1.8
        // clients only have 45 player-inventory slots. Sending all 46 entries
        // makes those clients address a slot that does not exist.
        let strip_offhand = *version < JavaMinecraftVersion::V_1_9
            && self.window_id.0 == 0
            && self.slot_data.len() == 46;
        let slot_count = self.slot_data.len() - usize::from(strip_offhand);

        if *version >= JavaMinecraftVersion::V_1_17_1 {
            let slot_count = i32::try_from(slot_count).map_err(|_| {
                WritingError::Message(format!("{slot_count} slot entries do not fit in VarInt"))
            })?;
            write.write_var_int(&VarInt(slot_count))?;
        } else {
            let slot_count = i16::try_from(slot_count).map_err(|_| {
                WritingError::Message(format!("{slot_count} slot entries do not fit in Short"))
            })?;
            write.write_i16_be(slot_count)?;
        }

        for (index, stack) in self.slot_data.iter().enumerate() {
            if strip_offhand && index == 45 {
                continue;
            }
            stack.write_with_version(&mut write, version)?;
        }
        if *version >= JavaMinecraftVersion::V_1_17_1 {
            self.carried_item.write_with_version(&mut write, version)?;
        }

        Ok(())
    }
}

impl<'a> ServerPacket<'a> for CSetContainerContent<'a> {
    fn read(bytebuf: &mut &'a [u8], version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        let window_id = bytebuf.get_container_id(version)?;
        let state_id = if *version >= JavaMinecraftVersion::V_1_17_1 {
            bytebuf.get_var_int()?
        } else {
            VarInt(0)
        };
        let count = if *version >= JavaMinecraftVersion::V_1_17_1 {
            bytebuf.get_var_int()?.0
        } else {
            i32::from(bytebuf.get_i16_be()?)
        };
        if !(0..=4096).contains(&count) {
            return Err(ReadingError::Message("Slot count out of bounds".into()));
        }
        let mut slot_data = Vec::with_capacity(count as usize);
        for _ in 0..count {
            slot_data.push(ItemStackSerializer::read_with_version(bytebuf, version)?);
        }
        let carried_item = if *version >= JavaMinecraftVersion::V_1_17_1 {
            ItemStackSerializer::read_with_version(bytebuf, version)?
        } else {
            ItemStackSerializer(std::borrow::Cow::Borrowed(
                pumpkin_data::item_stack::ItemStack::EMPTY,
            ))
        };
        Ok(Self {
            window_id,
            state_id,
            slot_data: Box::leak(slot_data.into_boxed_slice()),
            carried_item: Box::leak(Box::new(carried_item)),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::CSetContainerContent;
    use crate::codec::item_stack_seralizer::ItemStackSerializer;
    use crate::{ClientPacket, VarInt};
    use pumpkin_data::item::Item;
    use pumpkin_data::item_stack::ItemStack;
    use pumpkin_util::version::JavaMinecraftVersion;

    fn player_inventory_packet_with_offhand() -> (
        Vec<ItemStackSerializer<'static>>,
        ItemStackSerializer<'static>,
    ) {
        let stone = ItemStack::new(1, Item::from_id(1).expect("stone"));
        let mut slots: Vec<ItemStackSerializer> = (0..45)
            .map(|_| ItemStackSerializer::from(ItemStack::EMPTY.clone()))
            .collect();
        slots.push(ItemStackSerializer::from(stone));
        let carried = ItemStackSerializer::from(ItemStack::EMPTY.clone());
        (slots, carried)
    }

    #[test]
    fn pre_1_9_player_inventory_omits_offhand_slot() {
        let (slots, carried) = player_inventory_packet_with_offhand();
        let packet = CSetContainerContent::new(VarInt(0), VarInt(0), &slots, &carried);

        let mut out = Vec::new();
        packet
            .write_packet_data(&mut out, &JavaMinecraftVersion::V_1_8)
            .unwrap();

        // window id (u8) + slot count (i16) + 45 empty slots (i16 each).
        assert_eq!(out.len(), 1 + 2 + 45 * 2);
        assert_eq!(i16::from_be_bytes([out[1], out[2]]), 45);
    }

    #[test]
    fn post_1_9_player_inventory_keeps_offhand_slot() {
        let (slots, carried) = player_inventory_packet_with_offhand();
        let packet = CSetContainerContent::new(VarInt(0), VarInt(0), &slots, &carried);

        let mut out = Vec::new();
        packet
            .write_packet_data(&mut out, &JavaMinecraftVersion::V_1_9)
            .unwrap();

        // window id (u8) + slot count (i16) + 45 empty slots (i16 each) +
        // the non-empty offhand stack.
        assert_eq!(out.len(), 1 + 2 + 45 * 2 + 6);
        assert_eq!(i16::from_be_bytes([out[1], out[2]]), 46);
        assert_ne!(&out[out.len() - 6..out.len() - 4], &(-1i16).to_be_bytes());
    }

    #[test]
    fn legacy_serialization_does_not_mutate_offhand() {
        let (slots, carried) = player_inventory_packet_with_offhand();
        let packet = CSetContainerContent::new(VarInt(0), VarInt(0), &slots, &carried);

        for _ in 0..2 {
            let mut legacy = Vec::new();
            packet
                .write_packet_data(&mut legacy, &JavaMinecraftVersion::V_1_8)
                .unwrap();
            assert_eq!(i16::from_be_bytes([legacy[1], legacy[2]]), 45);
            assert!(!slots[45].0.as_ref().is_empty());

            let mut modern = Vec::new();
            packet
                .write_packet_data(&mut modern, &JavaMinecraftVersion::V_26_2)
                .unwrap();
            assert_eq!(&modern[48..52], &[0x01, 0x01, 0x00, 0x00]);
            assert!(!slots[45].0.as_ref().is_empty());
        }
    }
}
