use pumpkin_data::packet::serverbound::config::ACCEPT_CODE_OF_CONDUCT;
use pumpkin_macros::java_packet;

use crate::{ServerPacket, ser::ReadingError};

#[java_packet(ACCEPT_CODE_OF_CONDUCT)]
pub struct SAcceptCodeOfConduct;

impl<'a> ServerPacket<'a> for SAcceptCodeOfConduct {
    fn read(_bytebuf: &mut &'a [u8]) -> Result<Self, ReadingError> {
        Ok(Self)
    }
}

impl crate::ClientPacket for SAcceptCodeOfConduct {
    fn write_packet_data(
        &self,
        _write: impl std::io::Write,
    ) -> Result<(), crate::ser::WritingError> {
        Ok(())
    }
}
