use pumpkin_data::packet::clientbound::CONFIG_STORE_COOKIE;
use pumpkin_macros::java_packet;
use pumpkin_util::resource_location::ResourceLocation;

use crate::ClientPacket;
use crate::ser::NetworkWriteExt;
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(CONFIG_STORE_COOKIE)]
/// Stores some arbitrary data on the client, which persists between server transfers.
/// The Notchian (vanilla) client only accepts cookies of up to 5 KiB in size.
pub struct CStoreCookie<'a> {
    pub key: &'a ResourceLocation,
    pub payload: &'a [u8], // 5120,
}

impl<'a> CStoreCookie<'a> {
    #[must_use]
    pub const fn new(key: &'a ResourceLocation, payload: &'a [u8]) -> Self {
        Self { key, payload }
    }
}

impl ClientPacket for CStoreCookie<'_> {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_string(self.key)?;
        write.write_var_int(&crate::VarInt(self.payload.len() as i32))?;
        write.write_all(self.payload)?;
        Ok(())
    }
}
