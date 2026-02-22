use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

use crate::entity::player::Player;

use super::PlayerEvent;

/// An event that occurs when a player toggles flight.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerToggleFlightEvent {
    /// The player toggling flight.
    pub player: Arc<Player>,

    /// Whether the player is flying after the toggle.
    pub is_flying: bool,
}

impl PlayerToggleFlightEvent {
    /// Creates a new instance of `PlayerToggleFlightEvent`.
    pub const fn new(player: Arc<Player>, is_flying: bool) -> Self {
        Self {
            player,
            is_flying,
            cancelled: false,
        }
    }
}

impl PlayerEvent for PlayerToggleFlightEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
