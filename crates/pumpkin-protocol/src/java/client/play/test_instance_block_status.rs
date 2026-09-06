use crate::{
    ClientPacket, ServerPacket, VarInt,
    ser::{NetworkReadExt, NetworkReadSliceExt, NetworkWriteExt, ReadingError, WritingError},
};
use pumpkin_data::packet::clientbound::play::TEST_INSTANCE_BLOCK_STATUS;
use pumpkin_macros::java_packet;
use pumpkin_util::{math::position::BlockPos, version::JavaMinecraftVersion};

#[java_packet(TEST_INSTANCE_BLOCK_STATUS)]
pub struct CTestInstanceBlockStatus<'a> {
    pub pos: BlockPos,
    pub status: VarInt,
    pub message: Option<&'a str>,
}

impl<'a> CTestInstanceBlockStatus<'a> {
    #[must_use]
    pub const fn new(pos: BlockPos, status: VarInt, message: Option<&'a str>) -> Self {
        Self {
            pos,
            status,
            message,
        }
    }
}

impl ClientPacket for CTestInstanceBlockStatus<'_> {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        write.write_block_pos(&self.pos, version)?;
        write.write_var_int(&self.status)?;
        write.write_option(&self.message, |p, v| p.write_string(v))?;
        Ok(())
    }
}

impl<'a> ServerPacket<'a> for CTestInstanceBlockStatus<'a> {
    fn read(bytebuf: &mut &'a [u8], version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        Ok(Self {
            pos: bytebuf.get_block_pos(version)?,
            status: bytebuf.get_var_int()?,
            message: bytebuf.get_option(NetworkReadSliceExt::get_str_borrowed)?,
        })
    }
}
