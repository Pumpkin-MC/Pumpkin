use pumpkin_data::packet::clientbound::CONFIG_PING;
use pumpkin_macros::java_packet;

use crate::ClientPacket;
use crate::ser::NetworkWriteExt;
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(CONFIG_PING)]
pub struct CConfigPing {
    pub id: i32,
}

impl CConfigPing {
    #[must_use]
    pub const fn new(id: i32) -> Self {
        Self { id }
    }
}

impl ClientPacket for CConfigPing {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_i32_be(self.id)?;
        Ok(())
    }
}
