use pumpkin_data::packet::serverbound::play::CONFIGURATION_ACKNOWLEDGED;
use pumpkin_macros::java_packet;

use crate::{ServerPacket, ser::ReadingError};
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(CONFIGURATION_ACKNOWLEDGED)]
pub struct SConfigurationAcknowledged;

impl<'a> ServerPacket<'a> for SConfigurationAcknowledged {
    fn read(
        _bytebuf: &mut &'a [u8],
        _version: &JavaMinecraftVersion,
    ) -> Result<Self, ReadingError> {
        Ok(Self)
    }
}

impl crate::ClientPacket for SConfigurationAcknowledged {
    fn write_packet_data(
        &self,
        _write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        Ok(())
    }
}
