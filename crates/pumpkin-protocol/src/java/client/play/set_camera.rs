use pumpkin_data::packet::clientbound::play::SET_CAMERA;
use pumpkin_macros::java_packet;

use crate::{ClientPacket, codec::var_int::VarInt, ser::NetworkWriteExt};
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(SET_CAMERA)]
pub struct CSetCamera {
    pub camera_id: VarInt,
}

impl CSetCamera {
    #[must_use]
    pub const fn new(camera_id: VarInt) -> Self {
        Self { camera_id }
    }
}

impl ClientPacket for CSetCamera {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_var_int(&self.camera_id)?;
        Ok(())
    }
}
