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

/// Highlights the block targeted by `/test pos` with its structure-relative position.
#[java_packet(GAME_TEST_HIGHLIGHT_POS)]
pub struct CGameTestRelativePos {
    pub absolute_pos: BlockPos,
    pub relative_pos: BlockPos,
}

impl CGameTestRelativePos {
    #[must_use]
    pub const fn new(absolute_pos: BlockPos, relative_pos: BlockPos) -> Self {
        Self {
            absolute_pos,
            relative_pos,
        }
    }
}

impl ClientPacket for CGameTestRelativePos {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        write.write_block_pos(&self.absolute_pos, version)?;
        write.write_block_pos(&self.relative_pos, version)?;
        Ok(())
    }
}

impl<'a> ServerPacket<'a> for CGameTestRelativePos {
    fn read(bytebuf: &mut &'a [u8], version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        Ok(Self {
            absolute_pos: bytebuf.get_block_pos(version)?,
            relative_pos: bytebuf.get_block_pos(version)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relative_pos_uses_two_block_positions() -> Result<(), Box<dyn std::error::Error>> {
        let version = JavaMinecraftVersion::V_26_3;
        let packet = CGameTestRelativePos::new(BlockPos::new(1, 2, 3), BlockPos::new(4, 5, 6));
        let mut bytes = Vec::new();
        packet.write_packet_data(&mut bytes, &version)?;
        assert_eq!(bytes, [0, 0, 0, 64, 0, 0, 48, 2, 0, 0, 1, 0, 0, 0, 96, 5]);
        let decoded = CGameTestRelativePos::read(&mut bytes.as_slice(), &version)?;
        assert_eq!(decoded.absolute_pos, packet.absolute_pos);
        assert_eq!(decoded.relative_pos, packet.relative_pos);
        Ok(())
    }
}
