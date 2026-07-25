use pumpkin_data::packet::clientbound::PLAY_SET_BORDER_WARNING_DELAY;
use pumpkin_macros::java_packet;

use crate::ClientPacket;
use crate::VarInt;
use crate::ser::NetworkWriteExt;
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(PLAY_SET_BORDER_WARNING_DELAY)]
pub struct CSetBorderWarningDelay {
    pub warning_time: VarInt,
}

impl CSetBorderWarningDelay {
    #[must_use]
    pub const fn new(warning_time: VarInt) -> Self {
        Self { warning_time }
    }
}

impl ClientPacket for CSetBorderWarningDelay {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_var_int(&self.warning_time)?;
        Ok(())
    }
}
