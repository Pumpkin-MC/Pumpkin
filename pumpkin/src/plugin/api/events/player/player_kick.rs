use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

use crate::entity::player::Player;

use super::PlayerEvent;

/// An event that occurs when a player is kicked from the server.
///
/// If the event is cancelled, the player will not be kicked.
///
/// Matches Bukkit's `PlayerKickEvent`.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerKickEvent {
    /// The player being kicked.
    pub player: Arc<Player>,

    /// The reason for the kick.
    pub reason: String,
}

impl PlayerKickEvent {
    #[must_use]
    pub const fn new(player: Arc<Player>, reason: String) -> Self {
        Self {
            player,
            reason,
            cancelled: false,
        }
    }
}

impl PlayerEvent for PlayerKickEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
