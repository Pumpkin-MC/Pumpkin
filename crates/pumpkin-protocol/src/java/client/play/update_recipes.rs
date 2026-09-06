use crate::{
    ClientPacket, ServerPacket,
    ser::{NetworkReadSliceExt, NetworkWriteExt, ReadingError, WritingError},
};
use pumpkin_data::packet::clientbound::play::UPDATE_RECIPES;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(UPDATE_RECIPES)]
pub struct CUpdateRecipes<'a> {
    pub raw_data: &'a [u8],
}

impl<'a> CUpdateRecipes<'a> {
    #[must_use]
    pub const fn new(raw_data: &'a [u8]) -> Self {
        Self { raw_data }
    }
}

impl ClientPacket for CUpdateRecipes<'_> {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        write.write_slice(self.raw_data)?;
        Ok(())
    }
}

impl<'a> ServerPacket<'a> for CUpdateRecipes<'a> {
    fn read(bytebuf: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        let raw_data = bytebuf.read_remaining_slice_borrowed(usize::MAX)?;
        Ok(Self { raw_data })
    }
}
