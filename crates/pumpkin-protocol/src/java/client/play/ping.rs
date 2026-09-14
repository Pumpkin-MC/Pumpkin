use pumpkin_data::packet::clientbound::play::PING;
use pumpkin_macros::java_packet;

use crate::{ClientPacket, ser::NetworkWriteExt};
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(PING)]
pub struct CPlayPing {
    pub id: i32,
}

impl CPlayPing {
    #[must_use]
    pub const fn new(id: i32) -> Self {
        Self { id }
    }
}

impl ClientPacket for CPlayPing {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_i32_be(self.id)?;
        Ok(())
    }
}
