use pumpkin_data::packet::clientbound::PLAY_SET_BORDER_WARNING_DISTANCE;
use pumpkin_macros::java_packet;

use crate::ClientPacket;
use crate::VarInt;
use crate::ser::NetworkWriteExt;
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(PLAY_SET_BORDER_WARNING_DISTANCE)]
pub struct CSetBorderWarningDistance {
    pub warning_blocks: VarInt,
}

impl CSetBorderWarningDistance {
    #[must_use]
    pub const fn new(warning_blocks: VarInt) -> Self {
        Self { warning_blocks }
    }
}

impl ClientPacket for CSetBorderWarningDistance {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_var_int(&self.warning_blocks)?;
        Ok(())
    }
}
