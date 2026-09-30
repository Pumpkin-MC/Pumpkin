use crate::{
    ClientPacket, ServerPacket,
    ser::{NetworkReadExt, NetworkReadSliceExt, NetworkWriteExt, ReadingError, WritingError},
};
use pumpkin_data::packet::clientbound::play::GAME_TEST_HIGHLIGHT_POS;
use pumpkin_macros::java_packet;
use pumpkin_util::{math::position::BlockPos, version::JavaMinecraftVersion};

#[java_packet(GAME_TEST_HIGHLIGHT_POS)]
pub struct CGameTestHighlightPos<'a> {
    pub pos: BlockPos,
    pub color: i32,
    pub label: &'a str,
    pub duration_ms: i32,
}

impl<'a> CGameTestHighlightPos<'a> {
    #[must_use]
    pub const fn new(pos: BlockPos, color: i32, label: &'a str, duration_ms: i32) -> Self {
        Self {
            pos,
            color,
            label,
            duration_ms,
        }
    }
}

impl ClientPacket for CGameTestHighlightPos<'_> {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        write.write_block_pos(&self.pos, version)?;
        write.write_i32_be(self.color)?;
        write.write_string(self.label)?;
        write.write_i32_be(self.duration_ms)?;
        Ok(())
    }
}

impl<'a> ServerPacket<'a> for CGameTestHighlightPos<'a> {
    fn read(bytebuf: &mut &'a [u8], version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        Ok(Self {
            pos: bytebuf.get_block_pos(version)?,
            color: bytebuf.get_i32_be()?,
            label: bytebuf.get_str_borrowed()?,
            duration_ms: bytebuf.get_i32_be()?,
        })
    }
}
