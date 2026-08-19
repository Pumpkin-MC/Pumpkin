use pumpkin_data::packet::clientbound::PLAY_PLAYER_INFO_REMOVE;
use pumpkin_macros::java_packet;

use crate::ClientPacket;
use crate::VarInt;
use crate::ser::NetworkWriteExt;
use pumpkin_util::version::JavaMinecraftVersion;

/// Sent by the server to remove one or more players from the client's player list (tab list).
///
/// This packet is typically used when a player leaves the server or becomes invisible
/// to the recipient (e.g., moving out of tracking range).
#[java_packet(PLAY_PLAYER_INFO_REMOVE)]
pub struct CRemovePlayerInfo<'a> {
    /// A list of UUIDs corresponding to the players that should be removed.
    pub players: &'a [uuid::Uuid],
}

impl<'a> CRemovePlayerInfo<'a> {
    #[must_use]
    pub const fn new(players: &'a [uuid::Uuid]) -> Self {
        Self { players }
    }
}

impl ClientPacket for CRemovePlayerInfo<'_> {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_var_int(&VarInt(self.players.len() as i32))?;
        for uuid in self.players {
            write.write_uuid(uuid)?;
        }
        Ok(())
    }
}
