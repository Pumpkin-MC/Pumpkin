use pumpkin_data::packet::serverbound::config::PONG;
use pumpkin_macros::java_packet;

use crate::{ServerPacket, ser::NetworkReadExt, ser::ReadingError};
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(PONG)]
pub struct SConfigPong {
    pub id: i32,
}

impl<'a> ServerPacket<'a> for SConfigPong {
    fn read(bytebuf: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        Ok(Self {
            id: bytebuf.get_i32_be()?,
        })
    }
}

impl crate::ClientPacket for SConfigPong {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        use crate::ser::NetworkWriteExt;
        write.write_i32_be(self.id)?;
        Ok(())
    }
}
