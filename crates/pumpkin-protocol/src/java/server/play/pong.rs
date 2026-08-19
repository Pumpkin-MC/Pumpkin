use pumpkin_data::packet::serverbound::PLAY_PONG;
use pumpkin_macros::java_packet;

use crate::{
    ServerPacket,
    ser::{NetworkReadExt, ReadingError},
};
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(PLAY_PONG)]
pub struct SPlayPong {
    pub id: i32,
}

impl<'a> ServerPacket<'a> for SPlayPong {
    fn read(bytebuf: &mut &'a [u8], _version: &JavaMinecraftVersion) -> Result<Self, ReadingError> {
        Ok(Self {
            id: bytebuf.get_i32_be()?,
        })
    }
}

impl crate::ClientPacket for SPlayPong {
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
