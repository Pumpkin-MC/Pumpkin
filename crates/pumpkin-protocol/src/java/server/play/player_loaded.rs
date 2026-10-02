use crate::{ServerPacket, ser::ReadingError};
use pumpkin_data::packet::serverbound::play::PLAYER_LOADED;
use pumpkin_macros::java_packet;

/// Added in 1.21.4
#[java_packet(PLAYER_LOADED)]
pub struct SPlayerLoaded;

impl<'a> ServerPacket<'a> for SPlayerLoaded {
    fn read(_bytebuf: &mut &'a [u8]) -> Result<Self, ReadingError> {
        Ok(Self)
    }
}

impl crate::ClientPacket for SPlayerLoaded {
    fn write_packet_data(
        &self,
        _write: impl std::io::Write,
    ) -> Result<(), crate::ser::WritingError> {
        Ok(())
    }
}
