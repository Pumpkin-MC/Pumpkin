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
        let clicked_item = if *version >= JavaMinecraftVersion::V_1_21_5 {
            ItemStackSerializer::read_length_prefixed_optional(&mut read)?
        } else {
            ItemStackSerializer::read(&mut read)?
        };
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
