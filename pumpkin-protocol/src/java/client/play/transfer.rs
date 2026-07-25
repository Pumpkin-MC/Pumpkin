use crate::ClientPacket;
use crate::VarInt;
use crate::ser::NetworkWriteExt;
use pumpkin_data::packet::clientbound::PLAY_TRANSFER;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(PLAY_TRANSFER)]
pub struct CTransfer<'a> {
    pub host: &'a str,
    pub port: VarInt,
}

impl<'a> CTransfer<'a> {
    #[must_use]
    pub const fn new(host: &'a str, port: VarInt) -> Self {
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
        write.write_var_int(&self.port)?;
        Ok(())
    }
}
