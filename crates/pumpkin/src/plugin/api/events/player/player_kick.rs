use crate::entity::player::Player;
use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

/// An event that occurs when a player is kicked from the server.
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
