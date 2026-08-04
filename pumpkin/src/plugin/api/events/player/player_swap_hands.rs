use crate::entity::player::Player;
use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

/// An event that occurs when a player swaps items between main hand and offhand.
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
