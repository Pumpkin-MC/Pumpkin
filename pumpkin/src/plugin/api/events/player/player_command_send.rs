use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

use crate::entity::player::Player;

use super::PlayerEvent;

/// An event that occurs when a player executes a command
///
/// If the event is cancelled, the command will not be executed.
///
/// This event contains information about the player and the command being executed.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerCommandSendEvent {
    /// The player who is executing the command.
    pub player: Arc<Player>,

    /// The command being executed
    pub command: String,
}

impl PlayerCommandSendEvent {
    /// Creates a new instance of `PlayerCommandSendEvent`.
    ///
    /// # Arguments
    /// - `player`: A reference to the player running the command.
    /// - `command`: The command being executed.
    ///
    /// # Returns
    /// A new instance of `PlayerCommandSendEvent`.
    pub fn new(player: Arc<Player>, command: String) -> Self {
        Self {
            player,
            command,
            cancelled: false,
        }
    }
}

impl PlayerEvent for PlayerCommandSendEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
