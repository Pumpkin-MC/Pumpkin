use pumpkin_macros::{Event, cancellable};
use pumpkin_util::text::TextComponent;
use std::sync::Arc;

use crate::entity::player::Player;

use super::PlayerEvent;

/// An event that occurs when a player dies.
///
/// If the event is cancelled, the death is prevented (the player does not die).
///
/// This event contains information about the player and the death message.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerDeathEvent {
    /// The player who died.
    pub player: Arc<Player>,

    /// The death message to display to other players.
    pub death_message: TextComponent,

    /// Whether to keep the player's inventory on respawn.
    pub keep_inventory: bool,
}

impl PlayerDeathEvent {
    /// Creates a new instance of `PlayerDeathEvent`.
    ///
    /// # Arguments
    /// - `player`: A reference to the player who died.
    /// - `death_message`: The message to display upon death.
    ///
    /// # Returns
    /// A new instance of `PlayerDeathEvent`.
    pub const fn new(player: Arc<Player>, death_message: TextComponent) -> Self {
        Self {
            player,
            death_message,
            keep_inventory: false,
            cancelled: false,
        }
    }
}

impl PlayerEvent for PlayerDeathEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
