use pumpkin_data::packet::clientbound::PLAY_START_CONFIGURATION;
use pumpkin_macros::java_packet;

use crate::ClientPacket;
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(PLAY_START_CONFIGURATION)]
pub struct CStartConfiguration;

impl ClientPacket for CStartConfiguration {
    fn write_packet_data(
        &self,
        _write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        Ok(())
    }
}
