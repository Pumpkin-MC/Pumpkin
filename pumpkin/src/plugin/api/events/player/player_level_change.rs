use pumpkin_macros::Event;
use std::sync::Arc;

use crate::entity::player::Player;

use super::PlayerEvent;

/// An event that occurs when a player's experience level changes.
///
/// This event is not cancellable.
///
/// Matches Bukkit's `PlayerLevelChangeEvent`.
#[derive(Event, Clone)]
pub struct PlayerLevelChangeEvent {
    /// The player whose level is changing.
    pub player: Arc<Player>,

    /// The old experience level.
    pub old_level: i32,

    /// The new experience level.
    pub new_level: i32,
}

impl PlayerLevelChangeEvent {
    #[must_use]
    pub const fn new(player: Arc<Player>, old_level: i32, new_level: i32) -> Self {
        Self {
            player,
            old_level,
            new_level,
        }
    }
}

impl PlayerEvent for PlayerLevelChangeEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
