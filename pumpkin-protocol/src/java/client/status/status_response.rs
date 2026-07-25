use pumpkin_data::packet::clientbound::STATUS_STATUS_RESPONSE;
use pumpkin_macros::java_packet;

use crate::ClientPacket;
use crate::ser::NetworkWriteExt;
use pumpkin_util::version::JavaMinecraftVersion;

/// Sent by the server in response to a `SStatusRequest`.
///
/// This packet provides the client with the information required to display the
/// server in the multiplayer menu, including the MOTD, player count, and icon
#[java_packet(STATUS_STATUS_RESPONSE)]
pub struct CStatusResponse {
    /// A JSON-encoded string containing the server's status data.
    ///
    /// The maximum length of this string is 32,767 characters. It typically
    /// includes fields for `version`, `players`, `description` (MOTD), and `favicon`
    pub json_response: String,
}
impl CStatusResponse {
    #[must_use]
    pub const fn new(json_response: String) -> Self {
        Self { json_response }
    }
}

impl ClientPacket for CStatusResponse {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_string(&self.json_response)?;
        Ok(())
    }
}
