use pumpkin_macros::{Event, cancellable};
use pumpkin_util::text::TextComponent;
use std::sync::Arc;

use crate::entity::player::Player;

use super::PlayerEvent;

/// An event that occurs when a player is kicked from the server.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerKickEvent {
    /// The player being kicked.
    pub player: Arc<Player>,

    /// The kick reason shown to the player.
    pub reason: TextComponent,

    /// The leave message broadcasted to others.
    pub leave_message: TextComponent,

    /// The kick cause (best-effort).
    pub cause: String,
}

impl PlayerKickEvent {
    /// Creates a new instance of `PlayerKickEvent`.
    pub const fn new(
        player: Arc<Player>,
        reason: TextComponent,
        leave_message: TextComponent,
        cause: String,
    ) -> Self {
        Self {
            player,
            reason,
            leave_message,
            cause,
            cancelled: false,
        }
    }
}

impl PlayerEvent for PlayerKickEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
