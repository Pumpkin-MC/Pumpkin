use pumpkin_data::packet::clientbound::PLAY_LOW_DISK_SPACE_WARNING;
use pumpkin_macros::java_packet;

use crate::ClientPacket;
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(PLAY_LOW_DISK_SPACE_WARNING)]
pub struct CLowDiskSpaceWarning;

impl ClientPacket for CLowDiskSpaceWarning {
    fn write_packet_data(
        &self,
        _write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        Ok(())
    }
}
