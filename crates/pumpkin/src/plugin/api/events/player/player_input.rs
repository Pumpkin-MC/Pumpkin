use super::PlayerEvent;
use crate::entity::player::Player;
use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

/// An event that occurs when player inputs are received.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerInputEvent {
    /// The player.
    pub player: Arc<Player>,
    /// Input descriptor.
    pub input: String,
}

impl PlayerInputEvent {
    #[must_use]
    pub const fn new(player: Arc<Player>, input: String) -> Self {
        Self {
            player,
            input,
            cancelled: false,
        }
    }
}

impl PlayerEvent for PlayerInputEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
