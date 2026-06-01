use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

use crate::entity::player::Player;

use super::PlayerEvent;

/// An event that occurs when a player toggles their sprint state.
///
/// This event contains the player and their new sprint state.
/// It is cancellable to prevent the sprint state change.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerToggleSprintEvent {
    /// The player who toggled their sprint state.
    pub player: Arc<Player>,

    /// The new sprint state of the player.
    pub is_sprinting: bool,
}

impl PlayerToggleSprintEvent {
    /// Creates a new instance of `PlayerToggleSprintEvent`.
    ///
    /// # Arguments
    /// - `player`: A reference to the player who toggled their sprint state.
    /// - `is_sprinting`: The new sprint state.
    ///
    /// # Returns
    /// A new instance of `PlayerToggleSprintEvent`.
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
