use pumpkin_macros::Event;
use std::sync::Arc;

use crate::entity::player::Player;

use super::PlayerEvent;

/// An event that occurs when a player's experience level changes.
#[derive(Event, Clone)]
pub struct PlayerLevelChangeEvent {
    /// The player whose level changed.
    pub player: Arc<Player>,

    /// The previous experience level.
    pub old_level: i32,

    /// The new experience level.
    pub new_level: i32,
}

impl PlayerLevelChangeEvent {
    /// Creates a new instance of `PlayerLevelChangeEvent`.
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
