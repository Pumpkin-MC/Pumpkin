use pumpkin_data::packet::clientbound::play::BUNDLE_DELIMITER;
use pumpkin_macros::java_packet;

use crate::ClientPacket;

#[java_packet(BUNDLE_DELIMITER)]
pub struct CBundleDelimiter;

impl ClientPacket for CBundleDelimiter {
    fn write_packet_data(
        &self,
        _write: impl std::io::Write,
    ) -> Result<(), crate::ser::WritingError> {
        Ok(())
    }
}
