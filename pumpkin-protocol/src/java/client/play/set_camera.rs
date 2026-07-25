use pumpkin_data::packet::clientbound::PLAY_SET_CAMERA;
use pumpkin_util::version::JavaMinecraftVersion;

use crate::ClientPacket;
use crate::ser::NetworkWriteExt;
use crate::{VarInt, packet::MultiVersionJavaPacket};

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

impl MultiVersionJavaPacket for CSetCamera {
    fn to_id(version: JavaMinecraftVersion) -> i32 {
        PLAY_SET_CAMERA.to_id(version)
    }
}
