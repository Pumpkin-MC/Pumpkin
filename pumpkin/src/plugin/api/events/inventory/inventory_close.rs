use pumpkin_macros::Event;
use std::sync::Arc;

use crate::entity::player::Player;

/// An event that occurs when a player closes an inventory.
///
/// This event is not cancellable.
///
/// Matches Bukkit's `InventoryCloseEvent`.
#[derive(Event, Clone)]
pub struct InventoryCloseEvent {
    /// The player closing the inventory.
    pub player: Arc<Player>,

    /// The title of the inventory being closed.
    pub inventory_title: String,
}

impl InventoryCloseEvent {
    #[must_use]
    pub const fn new(player: Arc<Player>, inventory_title: String) -> Self {
        Self {
            player,
            inventory_title,
        }
    }
}
