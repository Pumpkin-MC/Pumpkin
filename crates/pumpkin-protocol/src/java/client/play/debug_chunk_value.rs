use crate::{
    ClientPacket, ServerPacket,
    ser::{NetworkReadExt, NetworkReadSliceExt, NetworkWriteExt, ReadingError, WritingError},
};
use pumpkin_data::packet::clientbound::play::DEBUG_CHUNK_VALUE;
use pumpkin_macros::java_packet;
use pumpkin_util::{math::vector2::Vector2, version::JavaMinecraftVersion};

#[java_packet(DEBUG_CHUNK_VALUE)]
pub struct CDebugChunkValue<'a> {
    pub chunk_pos: Vector2<i32>,
    pub name: &'a str,
    pub value: &'a str,
}

impl<'a> CDebugChunkValue<'a> {
    #[must_use]
    pub const fn new(chunk_pos: Vector2<i32>, name: &'a str, value: &'a str) -> Self {
        Self {
            chunk_pos,
            name,
            value,
        }
    }
}

impl ClientPacket for CDebugChunkValue<'_> {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        write.write_i32_be(self.chunk_pos.x)?;
        write.write_i32_be(self.chunk_pos.y)?;
        write.write_string(self.name)?;
        write.write_string(self.value)?;
        Ok(())
    }
}

impl<'a> ServerPacket<'a> for CDebugChunkValue<'a> {
    fn read(bytebuf: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        Ok(Self {
            chunk_pos: Vector2::new(bytebuf.get_i32_be()?, bytebuf.get_i32_be()?),
            name: bytebuf.get_str_borrowed()?,
            value: bytebuf.get_str_borrowed()?,
        })
    }
}
