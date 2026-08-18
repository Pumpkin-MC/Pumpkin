use pumpkin_data::packet::clientbound::PLAY_SET_BORDER_SIZE;
use pumpkin_macros::java_packet;

use crate::ClientPacket;
use crate::ser::NetworkWriteExt;
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(PLAY_SET_BORDER_SIZE)]
pub struct CSetBorderSize {
    pub diameter: f64,
}

impl CSetBorderSize {
    #[must_use]
    pub const fn new(diameter: f64) -> Self {
        Self { diameter }
    }
}

impl ClientPacket for CSetBorderSize {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_f64_be(self.diameter)?;
        Ok(())
    }
}
