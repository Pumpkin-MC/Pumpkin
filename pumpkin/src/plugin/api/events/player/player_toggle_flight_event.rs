use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

use crate::entity::player::Player;

use super::PlayerEvent;

/// An event that occurs when a player toggles their flight state.
///
/// This event contains the player and their new flight state.
/// It is cancellable to prevent the flight state change.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerToggleFlightEvent {
    /// The player who toggled their flight state.
    pub player: Arc<Player>,

    /// The new flight state of the player.
    pub is_flying: bool,
}

impl PlayerToggleFlightEvent {
    /// Creates a new instance of `PlayerToggleFlightEvent`.
    ///
    /// # Arguments
    /// - `player`: A reference to the player who toggled their flight state.
    /// - `is_flying`: The new flight state.
    ///
    /// # Returns
    /// A new instance of `PlayerToggleFlightEvent`.
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
