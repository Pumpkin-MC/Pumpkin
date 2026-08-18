use pumpkin_data::packet::clientbound::PLAY_OPEN_SIGN_EDITOR;
use pumpkin_macros::java_packet;
use pumpkin_util::math::position::BlockPos;

use crate::ClientPacket;
use crate::ser::NetworkWriteExt;
use pumpkin_util::version::JavaMinecraftVersion;

/// Opens the sign text input screen for the client.
///
/// This packet is sent by the server to force the client to show the
/// sign editing interface. This usually happens immediately after a
/// player places a sign or interacts with an existing one (if allowed).
#[java_packet(PLAY_OPEN_SIGN_EDITOR)]
pub struct COpenSignEditor {
    /// The world coordinates of the sign block to be edited.
    pub location: BlockPos,
    /// Whether the editor should open the front or the back of the sign.
    /// Introduced in the 1.20 "Trails & Tales" update for double-sided signs.
    pub is_front_text: bool,
}

impl COpenSignEditor {
    #[must_use]
    pub const fn new(location: BlockPos, is_front_text: bool) -> Self {
        Self {
            location,
            is_front_text,
        }
    }
}

impl ClientPacket for COpenSignEditor {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_block_pos(&self.location)?;
        write.write_bool(self.is_front_text)?;
        Ok(())
    }
}
