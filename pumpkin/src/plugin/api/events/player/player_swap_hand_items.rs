use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

use crate::entity::player::Player;

use super::PlayerEvent;

/// An event that occurs when a player swaps items between main hand and off hand.
///
/// If the event is cancelled, the item swap will not occur.
///
/// Matches Bukkit's `PlayerSwapHandItemsEvent`.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerSwapHandItemsEvent {
    /// The player swapping hand items.
    pub player: Arc<Player>,
}

impl PlayerSwapHandItemsEvent {
    #[must_use]
    pub const fn new(player: Arc<Player>) -> Self {
        Self {
            player,
            cancelled: false,
        }
    }
}

impl PlayerEvent for PlayerSwapHandItemsEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
