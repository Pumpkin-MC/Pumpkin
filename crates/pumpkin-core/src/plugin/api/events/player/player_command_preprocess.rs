use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

use super::PlayerEvent;
use crate::entity::player::Player;

/// An event that occurs before a player command is executed.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerCommandPreprocessEvent {
    /// The player sending the command.
    pub player: Arc<Player>,

    /// The raw command string.
    pub command: String,
}

impl PlayerCommandPreprocessEvent {
    #[must_use]
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
