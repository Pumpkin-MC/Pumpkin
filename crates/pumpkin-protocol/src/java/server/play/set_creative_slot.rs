use pumpkin_data::packet::serverbound::PLAY_SET_CREATIVE_MODE_SLOT;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

use crate::{
    ServerPacket,
    codec::item_stack_seralizer::ItemStackSerializer,
    ser::{NetworkReadExt, ReadingError},
};

#[java_packet(PLAY_SET_CREATIVE_MODE_SLOT)]
pub struct SSetCreativeSlot {
    pub slot: i16,
    pub clicked_item: ItemStackSerializer<'static>,
}

impl<'a> ServerPacket<'a> for SSetCreativeSlot {
    fn read(
        mut read: &mut &'a [u8],
        _version: &JavaMinecraftVersion,
    ) -> Result<Self, ReadingError> {
        let slot = read.get_i16_be()?;
        let clicked_item = ItemStackSerializer::read_length_prefixed_optional(&mut read)?;
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
        self.clicked_item.write_with_version(&mut write, version)?;
        Ok(())
    }
}
