use pumpkin_data::packet::clientbound::CONFIG_COOKIE_REQUEST;
use pumpkin_macros::java_packet;
use pumpkin_util::resource_location::ResourceLocation;

use crate::ClientPacket;
use crate::ser::NetworkWriteExt;
use pumpkin_util::version::JavaMinecraftVersion;

#[java_packet(CONFIG_COOKIE_REQUEST)]
/// Requests a cookie that was previously stored.
pub struct CCookieRequest<'a> {
    pub key: &'a ResourceLocation,
}

impl<'a> CCookieRequest<'a> {
    #[must_use]
    pub const fn new(key: &'a ResourceLocation) -> Self {
        Self { key }
    }
}

impl ClientPacket for CCookieRequest<'_> {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_string(self.key)?;
        Ok(())
    }
}
