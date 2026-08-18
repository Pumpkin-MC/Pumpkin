use crate::{
    ServerPacket,
    ser::{NetworkReadExt, NetworkReadSliceExt, ReadingError},
};
use pumpkin_data::packet::serverbound::PLAY_SET_COMMAND_BLOCK;
use pumpkin_macros::java_packet;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::version::JavaMinecraftVersion;

use crate::codec::var_int::VarInt;

#[java_packet(PLAY_SET_COMMAND_BLOCK)]
pub struct SSetCommandBlock<'a> {
    pub pos: BlockPos,
    pub command: &'a str,
    pub mode: VarInt,

    /// Operation mode flags
    /// - 0x01: Track output
    /// - 0x02: Is conditional
    /// - 0x04: Automatic
    pub flags: i8,
}

impl<'a> ServerPacket<'a> for SSetCommandBlock<'a> {
    fn read(bytebuf: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        Ok(Self {
            pos: BlockPos::from_i64(bytebuf.get_i64_be()?),
            command: bytebuf.get_str_bounded_borrowed(32767)?,
            mode: bytebuf.get_var_int()?,
            flags: bytebuf.get_i8()?,
        })
    }
}

impl crate::ClientPacket for SSetCommandBlock<'_> {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        use crate::ser::NetworkWriteExt;
        write.write_block_pos(&self.pos)?;
        write.write_string_bounded(self.command, 32767)?;
        write.write_var_int(&self.mode)?;
        write.write_i8(self.flags)?;
        Ok(())
    }
}

pub enum CommandBlockMode {
    Chain,
    Repeating,
    /// Redstone only
    Impulse,
}

impl TryFrom<VarInt> for CommandBlockMode {
    type Error = ();

    fn try_from(value: VarInt) -> Result<Self, Self::Error> {
        match value.0 {
            0 => Ok(Self::Chain),
            1 => Ok(Self::Repeating),
            2 => Ok(Self::Impulse),
            _ => Err(()),
        }
    }
}
