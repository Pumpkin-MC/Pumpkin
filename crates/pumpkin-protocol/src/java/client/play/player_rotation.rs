use pumpkin_data::packet::clientbound::play::PLAYER_ROTATION;
use pumpkin_macros::java_packet;

use crate::{ClientPacket, ser::NetworkWriteExt};

#[java_packet(PLAYER_ROTATION)]
pub struct CPlayerRotation {
    pub yaw: f32,
    pub pitch: f32,
}

impl CPlayerRotation {
    #[must_use]
    pub const fn new(yaw: f32, pitch: f32) -> Self {
        Self { yaw, pitch }
    }
}

impl ClientPacket for CPlayerRotation {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_f32_be(self.yaw)?;
        write.write_bool(false)?;
        write.write_f32_be(self.pitch)?;
        write.write_bool(false)?;
        Ok(())
    }
}
