use pumpkin_data::packet::serverbound::LOGIN_LOGIN_ACKNOWLEDGED;
use pumpkin_macros::java_packet;

use crate::{ServerPacket, ser::ReadingError};
use pumpkin_util::version::JavaMinecraftVersion;

/// Acknowledgement to the `CLoginSuccess` packet sent by the server.
#[java_packet(LOGIN_LOGIN_ACKNOWLEDGED)]
pub struct SLoginAcknowledged;

impl<'a> ServerPacket<'a> for SLoginAcknowledged {
    fn read(
        _bytebuf: &mut &'a [u8],
        _version: &JavaMinecraftVersion,
    ) -> Result<Self, ReadingError> {
        Ok(Self)
    }
}
