use pumpkin_data::packet::clientbound::play::OPEN_SIGN_EDITOR;
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
#[java_packet(OPEN_SIGN_EDITOR)]
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
        version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        if *version <= JavaMinecraftVersion::V_1_7_6 {
            write.write_i32_be(self.location.0.x)?;
            write.write_i32_be(self.location.0.y)?;
            write.write_i32_be(self.location.0.z)?;
        } else {
            write.write_block_pos(&self.location, version)?;
        }

        if *version >= JavaMinecraftVersion::V_1_20 {
            write.write_bool(self.is_front_text)?;
        }
        Ok(())
    }
}

impl<'a> crate::ServerPacket<'a> for COpenSignEditor {
    fn read(
        bytebuf: &mut &'a [u8],
        version: &JavaMinecraftVersion,
    ) -> Result<Self, crate::ser::ReadingError> {
        use crate::ser::NetworkReadExt;
        let location = if *version <= JavaMinecraftVersion::V_1_7_6 {
            let x = bytebuf.get_i32_be()?;
            let y = bytebuf.get_i32_be()?;
            let z = bytebuf.get_i32_be()?;
            BlockPos::new(x, y, z)
        } else {
            bytebuf.get_block_pos(version)?
        };

        let is_front_text = if *version >= JavaMinecraftVersion::V_1_20 {
            bytebuf.get_bool()?
        } else {
            true
        };

        Ok(Self {
            location,
            is_front_text,
        })
    }
}
