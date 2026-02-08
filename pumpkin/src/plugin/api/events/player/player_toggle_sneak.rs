use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

use crate::entity::player::Player;

use super::PlayerEvent;

/// An event that occurs when a player toggles sneaking.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerToggleSneakEvent {
    /// The player toggling sneaking.
    pub player: Arc<Player>,

    /// Whether the player is sneaking after the toggle.
    pub is_sneaking: bool,
}

impl PlayerToggleSneakEvent {
    /// Creates a new instance of `PlayerToggleSneakEvent`.
    pub const fn new(player: Arc<Player>, is_sneaking: bool) -> Self {
        Self {
            player,
            is_sneaking,
            cancelled: false,
        }
    }
}

impl PlayerEvent for PlayerToggleSneakEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
