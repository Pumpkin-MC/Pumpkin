use pumpkin_data::packet::serverbound::config::KEEP_ALIVE;
use pumpkin_macros::java_packet;

use crate::{
    ServerPacket,
    ser::{NetworkReadExt, ReadingError},
};
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(KEEP_ALIVE)]
pub struct SKeepAlive {
    pub keep_alive_id: i64,
}

impl<'a> ServerPacket<'a> for SKeepAlive {
    fn read(bytebuf: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        Ok(Self {
            keep_alive_id: bytebuf.get_i64_be()?,
        })
    }
}

impl crate::ClientPacket for SKeepAlive {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        use crate::ser::NetworkWriteExt;
        write.write_i64_be(self.keep_alive_id)?;
        Ok(())
    }
}
