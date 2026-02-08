use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

use crate::entity::player::Player;

/// An event that occurs when a player opens an inventory.
///
/// If the event is cancelled, the inventory will not be opened.
///
/// Matches Bukkit's `InventoryOpenEvent`.
#[cancellable]
#[derive(Event, Clone)]
pub struct InventoryOpenEvent {
    /// The player opening the inventory.
    pub player: Arc<Player>,

    /// The title of the inventory being opened.
    pub inventory_title: String,
}

impl InventoryOpenEvent {
    #[must_use]
    pub fn new(player: Arc<Player>, inventory_title: String) -> Self {
        Self {
            player,
            inventory_title,
            cancelled: false,
        }
    }
}
