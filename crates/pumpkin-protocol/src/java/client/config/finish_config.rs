use pumpkin_data::packet::clientbound::config::FINISH_CONFIGURATION;
use pumpkin_macros::java_packet;

use crate::ClientPacket;

#[java_packet(FINISH_CONFIGURATION)]
pub struct CFinishConfig;

impl ClientPacket for CFinishConfig {
    fn write_packet_data(
        &self,
        _write: impl std::io::Write,
    ) -> Result<(), crate::ser::WritingError> {
        Ok(())
    }
}

impl<'a> crate::ServerPacket<'a> for CFinishConfig {
    fn read(_bytebuf: &mut &'a [u8]) -> Result<Self, crate::ser::ReadingError> {
        Ok(Self)
    }
}
