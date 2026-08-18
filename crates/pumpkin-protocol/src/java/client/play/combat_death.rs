use crate::ClientPacket;
use crate::VarInt;
use crate::ser::NetworkWriteExt;
use pumpkin_data::packet::clientbound::PLAY_PLAYER_COMBAT_KILL;
use pumpkin_macros::java_packet;
use pumpkin_util::text::TextComponent;
use pumpkin_util::version::JavaMinecraftVersion;

/// Notifies the client that a player has died.
///
/// This packet is responsible for triggering the death screen on the client
/// and displaying the death message in the chat for the deceased player.
#[java_packet(PLAY_PLAYER_COMBAT_KILL)]
pub struct CCombatDeath<'a> {
    /// The Entity ID of the player who died.
    pub player_id: VarInt,
    /// The death message to be displayed (e.g., "Player was pricked to death by a Cactus").
    pub message: &'a TextComponent,
}

impl<'a> CCombatDeath<'a> {
    #[must_use]
    pub const fn new(player_id: VarInt, message: &'a TextComponent) -> Self {
        Self { player_id, message }
    }
}

impl ClientPacket for CCombatDeath<'_> {
    fn write_packet_data(
        &self,
        mut write: impl std::io::Write,
        _version: &JavaMinecraftVersion,
    ) -> Result<(), crate::ser::WritingError> {
        write.write_var_int(&self.player_id)?;
        write.write_slice(&self.message.encode())?;
        Ok(())
    }
}
