use pumpkin_data::packet::clientbound::PLAY_STORE_COOKIE;
use pumpkin_macros::java_packet;
use pumpkin_util::resource_location::ResourceLocation;

use crate::ClientPacket;
use crate::ser::NetworkWriteExt;
use pumpkin_util::version::JavaMinecraftVersion;

/// Stores some arbitrary data on the client, which persists between server transfers.
/// The Notchian client only accepts cookies of up to 5 kiB in size.
#[java_packet(PLAY_STORE_COOKIE)]
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
        write
            .write_all(self.payload)
            .map_err(|_| crate::ser::WritingError::Message("IO Error".into()))?;
        Ok(())
    }
}
