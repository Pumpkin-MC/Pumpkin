use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

use crate::entity::player::Player;

use super::PlayerEvent;

/// An event that occurs when a player toggles their sneak state.
///
/// This event contains the player and their new sneak state.
/// It is cancellable to prevent the sneak state change.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerToggleSneakEvent {
    /// The player who toggled their sneak state.
    pub player: Arc<Player>,

    /// The new sneak state of the player.
    pub is_sneaking: bool,
}

impl PlayerToggleSneakEvent {
    /// Creates a new instance of `PlayerToggleSneakEvent`.
    ///
    /// # Arguments
    /// - `player`: A reference to the player who toggled their sneak state.
    /// - `is_sneaking`: The new sneak state.
    ///
    /// # Returns
    /// A new instance of `PlayerToggleSneakEvent`.
    pub const fn new(player: Arc<Player>, is_sneaking: bool) -> Self {
        Self {
            player,
            is_sneaking,
            cancelled: false,
        }
    }
}

impl PlayerEvent for PlayerToggleSneakEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
