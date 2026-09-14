use pumpkin_data::packet::clientbound::play::CHUNKS_BIOMES;
use pumpkin_macros::java_packet;

use crate::{ClientPacket, codec::var_int::VarInt, ser::NetworkWriteExt};
use pumpkin_util::version::JavaMinecraftVersion;

pub struct ChunkBiomeEntry<'a> {
    pub chunk_x: i32,
    pub chunk_z: i32,
    pub data: &'a [u8],
}

#[java_packet(CHUNKS_BIOMES)]
pub struct CChunksBiomes<'a> {
    pub chunks: &'a [ChunkBiomeEntry<'a>],
}

impl<'a> CChunksBiomes<'a> {
    #[must_use]
    pub const fn new(chunks: &'a [ChunkBiomeEntry<'a>]) -> Self {
        Self { chunks }
    }
}

impl ClientPacket for CChunksBiomes<'_> {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_var_int(&VarInt(self.chunks.len() as i32))?;
        for chunk in self.chunks {
            write.write_i32_be(chunk.chunk_x)?;
            write.write_i32_be(chunk.chunk_z)?;
            write.write_slice(chunk.data)?;
        }
        Ok(())
    }
}
