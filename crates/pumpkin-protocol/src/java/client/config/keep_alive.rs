use crate::{
    ClientPacket, ServerPacket,
    ser::{NetworkReadExt, NetworkWriteExt, ReadingError, WritingError},
};
use pumpkin_data::packet::clientbound::config::KEEP_ALIVE;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(KEEP_ALIVE)]
pub struct CConfigKeepAlive {
    pub keep_alive_id: i64,
}

impl ClientPacket for CConfigKeepAlive {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), WritingError> {
        write.write_i64_be(self.keep_alive_id)?;
        Ok(())
    }
}

impl<'a> ServerPacket<'a> for CConfigKeepAlive {
    fn read(read: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        Ok(Self {
            keep_alive_id: read.get_i64_be()?,
        })
    }
}
