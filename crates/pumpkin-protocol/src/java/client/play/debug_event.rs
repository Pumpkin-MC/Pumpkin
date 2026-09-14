use crate::{
    ClientPacket, ServerPacket,
    ser::{NetworkReadSliceExt, NetworkWriteExt, ReadingError, WritingError},
};
use pumpkin_data::packet::clientbound::play::DEBUG_EVENT;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(DEBUG_EVENT)]
pub struct CDebugEvent<'a> {
    pub name: &'a str,
    pub data: &'a [u8],
}

impl<'a> CDebugEvent<'a> {
    #[must_use]
    pub const fn new(name: &'a str, data: &'a [u8]) -> Self {
        Self { name, data }
    }
}

impl ClientPacket for CDebugEvent<'_> {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        write.write_string(self.name)?;
        write.write_slice(self.data)?;
        Ok(())
    }
}

impl<'a> ServerPacket<'a> for CDebugEvent<'a> {
    fn read(bytebuf: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        let name = bytebuf.get_str_borrowed()?;
        let data = bytebuf.read_remaining_slice_borrowed(usize::MAX)?;
        Ok(Self { name, data })
    }
}
