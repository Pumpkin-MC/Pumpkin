use pumpkin_data::packet::clientbound::PLAY_PLAYER_COMBAT_KILL;
use pumpkin_macros::java_packet;
use pumpkin_util::text::TextComponent;
use serde::Serialize;

use crate::VarInt;

/// Notifies the client that a player has died.
///
/// This packet is responsible for triggering the death screen on the client
/// and displaying the death message in the chat for the deceased player.
#[derive(Serialize)]
#[java_packet(PLAY_PLAYER_COMBAT_KILL)]
pub struct CCombatDeath<'a> {
    /// The Entity ID of the player who died.
    pub player_id: VarInt,
    /// The death message to be displayed (e.g., "Player was pricked to death by a Cactus").
    pub message: &'a TextComponent,
}

impl<'a> CCombatDeath<'a> {
    #[must_use]
    pub fn new(player_id: VarInt, message: &'a TextComponent) -> Self {
        Self { player_id, message }
    }
}
