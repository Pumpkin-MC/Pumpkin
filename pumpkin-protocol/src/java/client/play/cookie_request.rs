use crate::ClientPacket;
use crate::ser::NetworkWriteExt;
use pumpkin_data::packet::clientbound::PLAY_COOKIE_REQUEST;
use pumpkin_macros::java_packet;
use pumpkin_util::resource_location::ResourceLocation;
use pumpkin_util::version::JavaMinecraftVersion;

/// Sent by the server to request a "cookie" (stored data) from the client.
///
/// Introduced in modern Minecraft versions, cookies allow servers to store
/// small amounts of persistent data on the client side that can be retrieved
/// even across different server instances or sub-servers in a network.
#[java_packet(PLAY_COOKIE_REQUEST)]
pub struct CPlayCookieRequest<'a> {
    /// The unique identifier (namespace:path) of the cookie to retrieve.
    pub key: &'a ResourceLocation,
}

impl<'a> CPlayCookieRequest<'a> {
    #[must_use]
    pub const fn new(key: &'a ResourceLocation) -> Self {
        Self { key }
    }
}

impl ClientPacket for CPlayCookieRequest<'_> {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_string(self.key)?;
        Ok(())
    }
}
