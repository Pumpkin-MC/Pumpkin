use pumpkin_data::packet::serverbound::PLAY_CONFIGURATION_ACKNOWLEDGED;
use pumpkin_macros::java_packet;

use crate::{ServerPacket, ser::ReadingError};
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(PLAY_CONFIGURATION_ACKNOWLEDGED)]
pub struct SConfigurationAcknowledged;

impl<'a> ServerPacket<'a> for SConfigurationAcknowledged {
    fn read(
        _bytebuf: &mut &'a [u8],
        _version: &JavaMinecraftVersion,
    ) -> Result<Self, ReadingError> {
        Ok(Self)
    }
}
