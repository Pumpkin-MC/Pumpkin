use pumpkin_data::packet::serverbound::play::PING_REQUEST;
use pumpkin_macros::java_packet;

use crate::{
    ServerPacket,
    ser::{NetworkReadExt, ReadingError},
};

#[java_packet(PING_REQUEST)]
pub struct SPlayPingRequest {
    pub payload: i64,
}

impl<'a> ServerPacket<'a> for SPlayPingRequest {
    fn read(bytebuf: &mut &'a [u8]) -> Result<Self, ReadingError> {
        Ok(Self {
            payload: bytebuf.get_i64_be()?,
        })
    }
}

impl crate::ClientPacket for SPlayPingRequest {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
    ) -> Result<(), crate::ser::WritingError> {
        use crate::ser::NetworkWriteExt;
        write.write_i64_be(self.payload)?;
        Ok(())
    }
}
