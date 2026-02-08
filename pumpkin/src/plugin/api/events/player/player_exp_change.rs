use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

use crate::entity::player::Player;

use super::PlayerEvent;

/// An event that occurs when a player's experience changes.
///
/// If the event is cancelled, the experience change will not occur.
///
/// Matches Bukkit's `PlayerExpChangeEvent`.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerExpChangeEvent {
    /// The player whose experience is changing.
    pub player: Arc<Player>,

    /// The amount of experience being gained or lost.
    pub amount: i32,
}

impl PlayerExpChangeEvent {
    #[must_use]
    pub const fn new(player: Arc<Player>, amount: i32) -> Self {
        Self {
            player,
            amount,
            cancelled: false,
        }
    }
}

impl PlayerEvent for PlayerExpChangeEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
