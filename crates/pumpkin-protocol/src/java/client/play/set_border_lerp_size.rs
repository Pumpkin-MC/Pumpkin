use pumpkin_data::packet::clientbound::PLAY_SET_BORDER_LERP_SIZE;
use pumpkin_macros::java_packet;

use crate::ClientPacket;
use crate::codec::var_long::VarLong;
use crate::ser::NetworkWriteExt;
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(PLAY_SET_BORDER_LERP_SIZE)]
pub struct CSetBorderLerpSize {
    pub old_diameter: f64,
    pub new_diameter: f64,
    pub speed: VarLong,
}

impl CSetBorderLerpSize {
    #[must_use]
    pub const fn new(old_diameter: f64, new_diameter: f64, speed: VarLong) -> Self {
        Self {
            old_diameter,
            new_diameter,
            speed,
        }
    }
}

impl ClientPacket for CSetBorderLerpSize {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_f64_be(self.old_diameter)?;
        write.write_f64_be(self.new_diameter)?;
        write.write_var_long(&self.speed)?;
        Ok(())
    }
}
