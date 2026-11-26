use pumpkin_data::packet::serverbound::play::CONFIGURATION_ACKNOWLEDGED;
use pumpkin_macros::java_packet;

use crate::{ServerPacket, ser::ReadingError};

#[java_packet(CONFIGURATION_ACKNOWLEDGED)]
pub struct SConfigurationAcknowledged;

impl<'a> ServerPacket<'a> for SConfigurationAcknowledged {
    fn read(_bytebuf: &mut &'a [u8]) -> Result<Self, ReadingError> {
        Ok(Self)
    }
}

impl crate::ClientPacket for SConfigurationAcknowledged {
    fn write_packet_data(
        &self,
        _write: impl std::io::Write,
    ) -> Result<(), crate::ser::WritingError> {
        Ok(())
    }
}
