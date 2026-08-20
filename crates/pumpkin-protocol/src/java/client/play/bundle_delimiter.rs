use pumpkin_data::packet::clientbound::PLAY_BUNDLE_DELIMITER;
use pumpkin_macros::java_packet;

use crate::ClientPacket;
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(PLAY_BUNDLE_DELIMITER)]
pub struct CBundleDelimiter;

impl ClientPacket for CBundleDelimiter {
    fn write_packet_data(
        &self,
        _write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        Ok(())
    }
}
