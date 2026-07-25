use pumpkin_data::packet::clientbound::CONFIG_FINISH_CONFIGURATION;
use pumpkin_macros::java_packet;

use crate::ClientPacket;
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(CONFIG_FINISH_CONFIGURATION)]
pub struct CFinishConfig;

impl ClientPacket for CFinishConfig {
    fn write_packet_data(
        &self,
        _write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        Ok(())
    }
}
