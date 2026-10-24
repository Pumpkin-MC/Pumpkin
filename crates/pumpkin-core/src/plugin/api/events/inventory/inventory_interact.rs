use crate::entity::player::Player;
use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

/// Base event triggered during inventory interaction.
#[cancellable]
#[derive(Event, Clone)]
pub struct InventoryInteractEvent {
    /// The player interacting with the inventory.
    pub player: Arc<Player>,
}

impl InventoryInteractEvent {
    #[must_use]
    pub const fn new(player: Arc<Player>) -> Self {
        Self {
            player,
            cancelled: false,
        }
    }
}
