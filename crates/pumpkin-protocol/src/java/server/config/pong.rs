use pumpkin_data::packet::serverbound::config::PONG;
use pumpkin_macros::java_packet;

use crate::{ServerPacket, ser::NetworkReadExt, ser::ReadingError};

#[java_packet(PONG)]
pub struct SConfigPong {
    pub id: i32,
}

impl<'a> ServerPacket<'a> for SConfigPong {
    fn read(bytebuf: &mut &'a [u8]) -> Result<Self, ReadingError> {
        Ok(Self {
            id: bytebuf.get_i32_be()?,
        })
    }
}

impl crate::ClientPacket for SConfigPong {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
    ) -> Result<(), crate::ser::WritingError> {
        use crate::ser::NetworkWriteExt;
        write.write_i32_be(self.id)?;
        Ok(())
    }
}
