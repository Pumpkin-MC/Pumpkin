use pumpkin_data::packet::clientbound::config::PING;
use pumpkin_macros::java_packet;

use crate::ClientPacket;
use crate::ser::NetworkWriteExt;
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(PING)]
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

impl<'a> crate::ServerPacket<'a> for CConfigPing {
    fn read(
        read: &mut &'a [u8],
        _version: &JavaMinecraftVersion,
    ) -> Result<Self, crate::ser::ReadingError> {
        use crate::ser::NetworkReadExt;
        Ok(Self {
            id: read.get_i32_be()?,
        })
    }
}
