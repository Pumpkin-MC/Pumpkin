use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

use crate::entity::player::Player;

use super::PlayerEvent;

/// An event that occurs when a player issues a command before it is processed.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerCommandPreprocessEvent {
    /// The player who sent the command.
    pub player: Arc<Player>,

    /// The raw command message, including the leading slash.
    pub command: String,
}

impl PlayerCommandPreprocessEvent {
    /// Creates a new instance of `PlayerCommandPreprocessEvent`.
    pub const fn new(player: Arc<Player>, command: String) -> Self {
        Self {
            player,
            command,
            cancelled: false,
        }
    }
}

impl PlayerEvent for PlayerCommandPreprocessEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
