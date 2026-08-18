use pumpkin_data::packet::clientbound::PLAY_DISCONNECT;
use pumpkin_util::text::TextComponent;

use crate::ClientPacket;
use crate::ser::NetworkWriteExt;
use pumpkin_macros::java_packet;
use pumpkin_util::version::JavaMinecraftVersion;

/// Forces the client to disconnect from the server while in the "Play" state.
///
/// This packet displays the provided reason to the player on a dedicated
/// disconnection screen. It is used for kicks, server shutdowns, or when
/// a player is banned.
#[java_packet(PLAY_DISCONNECT)]
pub struct CPlayDisconnect<'a> {
    /// The message shown to the player explaining why they were disconnected.
    /// This supports full JSON formatting (colors, bold, links, etc.).
    pub reason: &'a TextComponent,
}

impl<'a> CPlayDisconnect<'a> {
    #[must_use]
    pub const fn new(reason: &'a TextComponent) -> Self {
        Self { reason }
    }
}

impl ClientPacket for CPlayDisconnect<'_> {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_slice(&self.reason.encode())?;
        Ok(())
    }
}
