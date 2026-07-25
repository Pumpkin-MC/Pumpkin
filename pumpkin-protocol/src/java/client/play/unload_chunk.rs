use pumpkin_data::packet::clientbound::PLAY_FORGET_LEVEL_CHUNK;
use pumpkin_macros::java_packet;

use crate::ClientPacket;
use crate::ser::NetworkWriteExt;
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(PLAY_FORGET_LEVEL_CHUNK)]
pub struct CUnloadChunk {
    pub z: i32,
    pub x: i32,
}

impl CUnloadChunk {
    #[must_use]
    pub const fn new(x: i32, z: i32) -> Self {
        Self { z, x }
    }
}

impl ClientPacket for CUnloadChunk {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_i32_be(self.z)?;
        write.write_i32_be(self.x)?;
        Ok(())
    }
}
