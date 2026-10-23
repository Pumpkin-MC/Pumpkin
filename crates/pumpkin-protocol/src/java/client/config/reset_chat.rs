use pumpkin_data::packet::clientbound::config::RESET_CHAT;
use pumpkin_macros::java_packet;

use crate::ClientPacket;

#[java_packet(RESET_CHAT)]
pub struct CConfigResetChat;

impl ClientPacket for CConfigResetChat {
    fn write_packet_data(
        &self,
        _write: impl std::io::Write,
    ) -> Result<(), crate::ser::WritingError> {
        Ok(())
    }
}
