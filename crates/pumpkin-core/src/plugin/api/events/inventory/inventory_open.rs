use crate::entity::player::Player;
use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

/// An event that occurs when a player opens an inventory.
#[cancellable]
#[derive(Event, Clone)]
pub struct InventoryOpenEvent {
    /// The player opening the inventory.
    pub player: Arc<Player>,
}

impl InventoryOpenEvent {
    #[must_use]
    pub const fn new(player: Arc<Player>) -> Self {
        Self {
            player,
            cancelled: false,
        }
    }
}
