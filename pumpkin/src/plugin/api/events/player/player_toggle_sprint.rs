use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

use crate::entity::player::Player;

use super::PlayerEvent;

/// An event that occurs when a player toggles sprinting.
///
/// If the event is cancelled, the sprint state change will not occur.
///
/// Matches Bukkit's `PlayerToggleSprintEvent`.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerToggleSprintEvent {
    /// The player toggling sprint.
    pub player: Arc<Player>,

    /// Whether the player is now sprinting (`true`) or no longer sprinting (`false`).
    pub sprinting: bool,
}

impl PlayerToggleSprintEvent {
    #[must_use]
    pub const fn new(player: Arc<Player>, sprinting: bool) -> Self {
        Self {
            player,
            sprinting,
            cancelled: false,
        }
    }
}

impl PlayerEvent for PlayerToggleSprintEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
