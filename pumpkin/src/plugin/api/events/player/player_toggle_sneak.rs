use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

use crate::entity::player::Player;

use super::PlayerEvent;

/// An event that occurs when a player toggles sneaking.
///
/// If the event is cancelled, the sneak state change will not occur.
///
/// Matches Bukkit's `PlayerToggleSneakEvent`.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerToggleSneakEvent {
    /// The player toggling sneak.
    pub player: Arc<Player>,

    /// Whether the player is now sneaking (`true`) or no longer sneaking (`false`).
    pub sneaking: bool,
}

impl PlayerToggleSneakEvent {
    #[must_use]
    pub const fn new(player: Arc<Player>, sneaking: bool) -> Self {
        Self {
            player,
            sneaking,
            cancelled: false,
        }
    }
}

impl PlayerEvent for PlayerToggleSneakEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
