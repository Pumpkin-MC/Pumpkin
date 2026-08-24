use crate::{
    ServerPacket,
    ser::{NetworkReadExt, ReadingError},
};
use pumpkin_data::packet::serverbound::play::CHUNK_BATCH_RECEIVED;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(CHUNK_BATCH_RECEIVED)]
pub struct SChunkBatch {
    pub chunks_per_tick: f32,
}

impl<'a> ServerPacket<'a> for SChunkBatch {
    fn read(bytebuf: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        Ok(Self {
            chunks_per_tick: bytebuf.get_f32_be()?,
        })
    }
}

impl crate::ClientPacket for SChunkBatch {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        use crate::ser::NetworkWriteExt;
        write.write_f32_be(self.chunks_per_tick)?;
        Ok(())
    }
}
