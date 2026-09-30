use crate::{
    ClientPacket, ServerPacket, VarInt,
    ser::{NetworkReadExt, NetworkWriteExt, ReadingError, WritingError},
};
use pumpkin_data::packet::clientbound::play::SET_CHUNK_CACHE_RADIUS;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(SET_CHUNK_CACHE_RADIUS)]
pub struct CSetChunkCacheRadius {
    pub radius: VarInt,
}

impl CSetChunkCacheRadius {
    #[must_use]
    pub const fn new(radius: VarInt) -> Self {
        Self { radius }
    }
}

impl ClientPacket for CSetChunkCacheRadius {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        write.write_var_int(&self.radius)?;
        Ok(())
    }
}

impl<'a> ServerPacket<'a> for CSetChunkCacheRadius {
    fn read(bytebuf: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        Ok(Self {
            radius: bytebuf.get_var_int()?,
        })
    }
}
