use crate::{
    ClientPacket, ServerPacket,
    ser::{NetworkReadExt, NetworkReadSliceExt, NetworkWriteExt, ReadingError, WritingError},
};
use pumpkin_data::packet::clientbound::play::DEBUG_BLOCK_VALUE;
use pumpkin_macros::java_packet;
use pumpkin_util::{math::position::BlockPos, version::JavaMinecraftVersion};

#[java_packet(DEBUG_BLOCK_VALUE)]
pub struct CDebugBlockValue<'a> {
    pub pos: BlockPos,
    pub name: &'a str,
    pub value: &'a str,
}

impl<'a> CDebugBlockValue<'a> {
    #[must_use]
    pub const fn new(pos: BlockPos, name: &'a str, value: &'a str) -> Self {
        Self { pos, name, value }
    }
}

impl ClientPacket for CDebugBlockValue<'_> {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        write.write_block_pos(&self.pos, version)?;
        write.write_string(self.name)?;
        write.write_string(self.value)?;
        Ok(())
    }
}

impl<'a> ServerPacket<'a> for CDebugBlockValue<'a> {
    fn read(bytebuf: &mut &'a [u8], version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        Ok(Self {
            pos: bytebuf.get_block_pos(version)?,
            name: bytebuf.get_str_borrowed()?,
            value: bytebuf.get_str_borrowed()?,
        })
    }
}
