use pumpkin_data::packet::clientbound::PLAY_PONG_RESPONSE;
use pumpkin_macros::java_packet;

use crate::ClientPacket;
use crate::ser::NetworkWriteExt;
use pumpkin_util::version::JavaMinecraftVersion;

/// Responds to a client-initiated ping request to synchronize game state.
#[java_packet(PLAY_PONG_RESPONSE)]
pub struct CPingResponse {
    /// The unique identifier sent by the client in the initial Ping packet.
    /// The server must return this exact value.
    pub payload: i64,
}

impl CPingResponse {
    #[must_use]
    pub const fn new(payload: i64) -> Self {
        Self { payload }
    }
}

impl ClientPacket for CPingResponse {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_i64_be(self.payload)?;
        Ok(())
    }
}
