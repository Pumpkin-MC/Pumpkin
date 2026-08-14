use crate::VarInt;
use pumpkin_data::packet::clientbound::CONFIG_TRANSFER;
use pumpkin_macros::java_packet;

use crate::ClientPacket;
use crate::ser::NetworkWriteExt;
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(CONFIG_TRANSFER)]
pub struct CTransfer<'a> {
    pub host: &'a str,
    pub port: &'a VarInt,
}

impl<'a> CTransfer<'a> {
    #[must_use]
    pub const fn new(host: &'a str, port: &'a VarInt) -> Self {
        Self { host, port }
    }
}

impl ClientPacket for CTransfer<'_> {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_string(self.host)?;
        write.write_var_int(self.port)?;
        Ok(())
    }
}
