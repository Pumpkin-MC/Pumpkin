use crate::entity::player::Player;
use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

/// An event that occurs when a player drags items across inventory slots.
#[cancellable]
#[derive(Event, Clone)]
pub struct InventoryDragEvent {
    /// The player dragging items.
    pub player: Arc<Player>,
}

impl InventoryDragEvent {
    #[must_use]
    pub const fn new(player: Arc<Player>) -> Self {
        Self {
            player,
            cancelled: false,
        }
    }
}
