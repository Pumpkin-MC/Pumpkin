use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

use crate::entity::player::Player;

use super::PlayerEvent;

/// An event that occurs when a player toggles sprinting.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerToggleSprintEvent {
    /// The player toggling sprinting.
    pub player: Arc<Player>,

    /// Whether the player is sprinting after the toggle.
    pub is_sprinting: bool,
}

impl PlayerToggleSprintEvent {
    /// Creates a new instance of `PlayerToggleSprintEvent`.
    pub const fn new(player: Arc<Player>, is_sprinting: bool) -> Self {
        Self {
            player,
            is_sprinting,
            cancelled: false,
        }
    }
}

impl PlayerEvent for PlayerToggleSprintEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
