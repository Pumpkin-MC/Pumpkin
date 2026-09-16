use pumpkin_data::packet::serverbound::play::SET_CREATIVE_MODE_SLOT;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

use crate::{
    ServerPacket,
    codec::item_stack_seralizer::ItemStackSerializer,
    ser::{NetworkReadExt, ReadingError},
};

#[java_packet(SET_CREATIVE_MODE_SLOT)]
pub struct SSetCreativeSlot {
    pub slot: i16,
    pub clicked_item: ItemStackSerializer<'static>,
}

impl SSetCreativeSlot {
    #[must_use]
    pub const fn new(slot: i16, clicked_item: ItemStackSerializer<'static>) -> Self {
        Self { slot, clicked_item }
    }
}

impl<'a> ServerPacket<'a> for SSetCreativeSlot {
    fn read(mut read: &mut &'a [u8], version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        let slot = if *version >= JavaMinecraftVersion::V_1_20_5 {
            read.get_u16_be()? as i16
        } else {
            read.get_i16_be()?
        };
        let clicked_item = ItemStackSerializer::read_untrusted_with_version(&mut read, version)?;
        Ok(Self { slot, clicked_item })
    }
}

impl crate::ClientPacket for SSetCreativeSlot {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        use crate::ser::NetworkWriteExt;
        write.write_i16_be(self.slot)?;
        if *version >= JavaMinecraftVersion::V_1_21_5 {
            self.clicked_item
                .write_length_prefixed_with_version(&mut write, version)?;
        } else {
            self.clicked_item.write_with_version(&mut write, version)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::SSetCreativeSlot;
    use crate::ServerPacket;
    use crate::ser::NetworkWriteExt;
    use pumpkin_data::item::Item;
    use pumpkin_data::item_id_remap::remap_item_id_for_version;
    use pumpkin_util::version::JavaMinecraftVersion;

    #[test]
    fn reads_26_3_stack_once_remapped() {
        let version = JavaMinecraftVersion::V_26_3;
        let mut buf = Vec::new();
        buf.write_u16_be(36).unwrap();
        buf.write_var_int(&1.into()).unwrap();
        let item_id = remap_item_id_for_version(Item::DIAMOND_PICKAXE.id, version);
        buf.write_var_int(&i32::from(item_id).into()).unwrap();
        buf.write_var_int(&2.into()).unwrap();
        buf.write_var_int(&0.into()).unwrap();
        // `damage` (3) = 17
        buf.write_slice(&[3, 1, 17]).unwrap();
        // `waxed` (120) only exists in 26.3 and is skipped
        buf.write_slice(&[120, 0]).unwrap();

        let packet = SSetCreativeSlot::read(&mut buf.as_slice(), &version).unwrap();
        let stack = packet.clicked_item.to_stack();
        assert_eq!(stack.item.id, Item::DIAMOND_PICKAXE.id);
        assert_eq!(stack.get_damage(), 17);
        assert_eq!(stack.patch.len(), 1);
    }
}
