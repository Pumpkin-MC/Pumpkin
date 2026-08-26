use std::io::Write;

use crate::VarInt;
use crate::codec::item_stack_seralizer::ItemStackSerializer;
use crate::{ClientPacket, WritingError, ser::NetworkWriteExt};

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

        // The player inventory screen handler (window id 0) always includes the
        // offhand slot at index 45. Offhand was introduced in 1.9, so clients
        // older than that only expect 45 slots and would reject a 46-slot window
        // 0 payload with `IndexOutOfBoundsException: Index: 45, Size: 45`.
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

#[cfg(test)]
mod tests {
    use super::CSetContainerContent;
    use crate::codec::item_stack_seralizer::ItemStackSerializer;
    use crate::{ClientPacket, VarInt};
    use pumpkin_data::item::Item;
    use pumpkin_data::item_stack::ItemStack;
    use pumpkin_util::version::JavaMinecraftVersion;

    /// A 46-slot player inventory where the offhand slot (index 45) holds a
    /// non-empty stone stack and every other slot is empty.
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
        // The non-empty offhand stack must be omitted entirely.
        assert_eq!(out.len(), 1 + 2 + 45 * 2);
        let count = i16::from_be_bytes([out[1], out[2]]);
        assert_eq!(count, 45);
    }

    #[test]
    fn post_1_9_player_inventory_keeps_offhand_slot() {
        let (slots, carried) = player_inventory_packet_with_offhand();
        let packet = CSetContainerContent::new(VarInt(0), VarInt(0), &slots, &carried);

        let mut out = Vec::new();
        packet
            .write_packet_data(&mut out, &JavaMinecraftVersion::V_1_9)
            .unwrap();

        // window id (u8) + slot count (i16) + 45 empty slots (i16 each) + the
        // non-empty offhand stack (i16 item id, i8 count, i16 damage, u8 NBT).
        assert_eq!(out.len(), 1 + 2 + 45 * 2 + 6);
        let count = i16::from_be_bytes([out[1], out[2]]);
        assert_eq!(count, 46);
        // The last slot is the offhand and must not be the empty-item marker (-1).
        assert_ne!(&out[out.len() - 6..out.len() - 4], &(-1i16).to_be_bytes());
    }

    #[test]
    fn offhand_survives_alternating_legacy_and_modern_joins() {
        let (slots, carried) = player_inventory_packet_with_offhand();
        let packet = CSetContainerContent::new(VarInt(0), VarInt(0), &slots, &carried);

        // Sequence: 1.8 -> 26.2 -> 1.8 -> 26.2. Serialization is read-only, so
        // the server-side offhand stack must never be mutated, and every modern
        // (26.2) join must still encode the offhand item normally.
        for _ in 0..2 {
            // 1.8 join: the offhand slot is stripped from the wire.
            let mut legacy = Vec::new();
            packet
                .write_packet_data(&mut legacy, &JavaMinecraftVersion::V_1_8)
                .unwrap();
            assert_eq!(legacy.len(), 1 + 2 + 45 * 2);
            assert_eq!(i16::from_be_bytes([legacy[1], legacy[2]]), 45);
            assert!(!slots[45].0.as_ref().is_empty());

            // 26.2 join: the offhand item must still be present and encoded.
            let mut modern = Vec::new();
            packet
                .write_packet_data(&mut modern, &JavaMinecraftVersion::V_26_2)
                .unwrap();
            // window id (VarInt) + state id (VarInt) + slot count (VarInt) +
            // 45 empty slots (VarInt 0) + non-empty offhand (count, item id, add
            // count, remove count as VarInts) + empty carried item (VarInt 0).
            assert_eq!(modern.len(), 1 + 1 + 1 + 45 + 4 + 1);
            // offhand: count 1, stone id 1, 0 components added, 0 removed.
            assert_eq!(&modern[48..52], &[0x01, 0x01, 0x00, 0x00]);
            assert!(!slots[45].0.as_ref().is_empty());
        }
    }
}
