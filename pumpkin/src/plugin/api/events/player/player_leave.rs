use pumpkin_macros::{Event, cancellable};
use pumpkin_util::text::TextComponent;
use std::sync::Arc;

use crate::entity::player::Player;

use super::PlayerEvent;

/// An event that occurs when a player leaves the game.
///
/// This event contains information about the player leaving and a message to display upon leaving.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerLeaveEvent {
    /// The player who is leaving the game.
    pub player: Arc<Player>,

    /// The message to display when the player leaves.
    pub leave_message: TextComponent,
}

impl PlayerLeaveEvent {
    /// Creates a new instance of `PlayerLeaveEvent`.
    ///
    /// # Arguments
    /// - `player`: A reference to the player leaving the game.
    /// - `leave_message`: The message to display upon leaving.
    ///
    /// # Returns
    /// A new instance of `PlayerLeaveEvent`.
    pub fn new(player: Arc<Player>, leave_message: TextComponent) -> Self {
        Self {
            player,
            leave_message,
            cancelled: false,
        }
    }
}

impl PlayerEvent for PlayerLeaveEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
