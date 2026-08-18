use crate::{ServerPacket, ser::ReadingError};
use pumpkin_data::packet::serverbound::PLAY_CLIENT_TICK_END;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(PLAY_CLIENT_TICK_END)]
pub struct SClientTickEnd;

impl<'a> ServerPacket<'a> for SClientTickEnd {
    fn read(
        _bytebuf: &mut &'a [u8],
        _protocol_version: &JavaMinecraftVersion,
    ) -> Result<Self, ReadingError> {
        Ok(Self)
    }
}

impl crate::ClientPacket for SClientTickEnd {
    fn write_packet_data(
        &self,
        _write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        Ok(())
    }
}
