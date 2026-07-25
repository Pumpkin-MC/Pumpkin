use pumpkin_data::packet::clientbound::CONFIG_CLEAR_DIALOG;
use pumpkin_macros::java_packet;

use crate::ClientPacket;
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(CONFIG_CLEAR_DIALOG)]
pub struct CConfigClearDialog;

impl CConfigClearDialog {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Default for CConfigClearDialog {
    fn default() -> Self {
        Self::new()
    }
}

impl ClientPacket for CConfigClearDialog {
    fn write_packet_data(
        &self,
        _write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        Ok(())
    }
}
