use crate::entity::player::Player;
use pumpkin_macros::Event;
use std::sync::Arc;

/// An event that occurs when a player's experience level changes.
#[derive(Event, Clone)]
pub struct PlayerLevelChangeEvent {
    /// The player whose level changed.
    pub player: Arc<Player>,

    /// The old level.
    pub old_level: i32,

    /// The new level.
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
