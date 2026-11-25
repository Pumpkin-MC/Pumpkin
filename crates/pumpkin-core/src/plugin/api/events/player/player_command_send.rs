use pumpkin_macros::Event;
use std::sync::Arc;

use crate::entity::player::Player;

use super::PlayerEvent;

/// Fired when the list of available server commands is sent to a player.
///
/// Blocking handlers may remove entries from `commands` to hide them from the
/// client's command graph; added entries are ignored.
#[derive(Event, Clone)]
pub struct PlayerCommandSendEvent {
    pub player: Arc<Player>,
    pub commands: Vec<String>,
}

impl PlayerCommandSendEvent {
    #[must_use]
    pub const fn new(player: Arc<Player>, commands: Vec<String>) -> Self {
        Self { player, commands }
    }
}

impl PlayerEvent for PlayerCommandSendEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
