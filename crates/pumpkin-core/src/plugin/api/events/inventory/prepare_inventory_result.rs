use crate::entity::player::Player;
use pumpkin_macros::Event;
use std::sync::Arc;

/// Generalized event that occurs when an inventory result slot item is prepared.
#[derive(Event, Clone)]
pub struct PrepareInventoryResultEvent {
    /// The player interacting with the inventory.
    pub player: Arc<Player>,

    /// The result item ID, if any.
    pub result_item: Option<String>,
}

impl PrepareInventoryResultEvent {
    #[must_use]
    pub const fn new(player: Arc<Player>, result_item: Option<String>) -> Self {
        Self {
            player,
            result_item,
        }
    }
}
