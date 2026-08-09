use crate::{ServerPacket, ser::ReadingError};
use pumpkin_data::packet::serverbound::PLAY_PLAYER_LOADED;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(PLAY_PLAYER_LOADED)]
pub struct SPlayerLoaded;

impl<'a> ServerPacket<'a> for SPlayerLoaded {
    fn read(
        _bytebuf: &mut &'a [u8],
        _protocol_version: &JavaMinecraftVersion,
    ) -> Result<Self, ReadingError> {
        Ok(Self)
    }
}
