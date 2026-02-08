use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

use crate::entity::player::Player;

use super::PlayerEvent;

/// An event that occurs when a player toggles flight.
///
/// If the event is cancelled, the flight state change will not occur.
///
/// Matches Bukkit's `PlayerToggleFlightEvent`.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerToggleFlightEvent {
    /// The player toggling flight.
    pub player: Arc<Player>,

    /// Whether the player is now flying (`true`) or no longer flying (`false`).
    pub flying: bool,
}

impl PlayerToggleFlightEvent {
    #[must_use]
    pub const fn new(player: Arc<Player>, flying: bool) -> Self {
        Self {
            player,
            flying,
            cancelled: false,
        }
    }
}

impl PlayerEvent for PlayerToggleFlightEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
