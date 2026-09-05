use crate::{
    ServerPacket,
    ser::{NetworkReadExt, ReadingError},
};
use pumpkin_data::packet::serverbound::play::JIGSAW_GENERATE;
use pumpkin_macros::java_packet;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::version::JavaMinecraftVersion;

use crate::codec::var_int::VarInt;

#[java_packet(JIGSAW_GENERATE)]
pub struct SJigsawGenerate {
    pub pos: BlockPos,
    pub levels: VarInt,
    pub keep_jigsaws: bool,
}

impl<'a> ServerPacket<'a> for SJigsawGenerate {
    fn read(bytebuf: &mut &'a [u8], version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        Ok(Self {
            pos: bytebuf.get_block_pos(version)?,
            levels: bytebuf.get_var_int()?,
            keep_jigsaws: bytebuf.get_bool()?,
        })
    }
}

impl crate::ClientPacket for SJigsawGenerate {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        use crate::ser::NetworkWriteExt;
        write.write_block_pos(&self.pos, version)?;
        write.write_var_int(&self.levels)?;
        write.write_bool(self.keep_jigsaws)?;
        Ok(())
    }
}
