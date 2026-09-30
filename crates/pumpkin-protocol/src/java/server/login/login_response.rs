use pumpkin_data::packet::serverbound::login::LOGIN_ACKNOWLEDGED;
use pumpkin_macros::java_packet;

use crate::{ServerPacket, ser::ReadingError};
use pumpkin_util::version::JavaMinecraftVersion;

/// Acknowledgement to the `CLoginSuccess` packet sent by the server.
#[java_packet(LOGIN_ACKNOWLEDGED)]
pub struct SLoginAcknowledged;

impl<'a> ServerPacket<'a> for SLoginAcknowledged {
    fn read(
        _bytebuf: &mut &'a [u8],
        _version: &JavaMinecraftVersion,
    ) -> Result<Self, ReadingError> {
        Ok(Self)
    }
}

impl crate::ClientPacket for SLoginAcknowledged {
    fn write_packet_data(
        &self,
        _write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        Ok(())
    }
}
