use crate::{
    ServerPacket,
    ser::{NetworkReadSliceExt, ReadingError},
};
use pumpkin_data::packet::serverbound::play::RENAME_ITEM;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

#[derive(Debug)]
#[java_packet(RENAME_ITEM)]
pub struct SRenameItem<'a> {
    pub item_name: &'a str,
}

impl<'a> ServerPacket<'a> for SRenameItem<'a> {
    fn read(bytebuf: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        Ok(Self {
            item_name: bytebuf.get_str_bounded_borrowed(32767)?,
        })
    }
}

impl crate::ClientPacket for SRenameItem<'_> {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        use crate::ser::NetworkWriteExt;
        write.write_string_bounded(self.item_name, 32767)?;
        Ok(())
    }
}
