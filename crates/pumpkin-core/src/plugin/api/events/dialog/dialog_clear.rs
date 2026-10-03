use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

use crate::entity::player::Player;

use super::super::player::PlayerEvent;

/// An event that occurs when a dialog is cleared for a player.
#[cancellable]
#[derive(Event, Clone)]
pub struct DialogClearEvent {
    /// The player whose dialog is being cleared.
    pub player: Arc<Player>,
}

impl DialogClearEvent {
    #[must_use]
    pub const fn new(player: Arc<Player>) -> Self {
        Self {
            player,
            cancelled: false,
        }
    }
}

impl PlayerEvent for DialogClearEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
