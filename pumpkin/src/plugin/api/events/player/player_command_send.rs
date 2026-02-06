use pumpkin_macros::Event;
use std::sync::Arc;

use crate::entity::player::Player;

use super::PlayerEvent;

/// An event that occurs when the server sends the available command list to a player.
#[derive(Event, Clone)]
pub struct PlayerCommandSendEvent {
    /// The player receiving the command list.
    pub player: Arc<Player>,

    /// The commands that will be sent to the player.
    pub commands: Vec<String>,
}

impl PlayerCommandSendEvent {
    /// Creates a new instance of `PlayerCommandSendEvent`.
    pub const fn new(player: Arc<Player>, commands: Vec<String>) -> Self {
        Self { player, commands }
    }
}

impl PlayerEvent for PlayerCommandSendEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
