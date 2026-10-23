use crate::{ServerPacket, ser::ReadingError};
use pumpkin_data::packet::serverbound::play::CLIENT_TICK_END;
use pumpkin_macros::java_packet;

#[java_packet(CLIENT_TICK_END)]
pub struct SClientTickEnd;

impl<'a> ServerPacket<'a> for SClientTickEnd {
    fn read(_bytebuf: &mut &'a [u8]) -> Result<Self, ReadingError> {
        Ok(Self)
    }
}

impl crate::ClientPacket for SClientTickEnd {
    fn write_packet_data(
        &self,
        _write: impl std::io::Write,
    ) -> Result<(), crate::ser::WritingError> {
        Ok(())
    }
}
