use pumpkin_data::packet::clientbound::play::LOW_DISK_SPACE_WARNING;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

use crate::ClientPacket;

/// Warns the client that the server is running low on disk space.
///
/// Added in 26.1. This packet informs the client to display a warning
/// overlay/notification about low storage.
#[java_packet(LOW_DISK_SPACE_WARNING)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CLowDiskSpaceWarning;

impl CLowDiskSpaceWarning {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl ClientPacket for CLowDiskSpaceWarning {
    fn write_packet_data(
        &self,
        _write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        Ok(())
    }
}
