use crate::entity::player::Player;
use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

/// An event that occurs when a creative mode player sets an inventory slot.
#[cancellable]
#[derive(Event, Clone)]
pub struct InventoryCreativeEvent {
    /// The player taking creative action.
    pub player: Arc<Player>,

    /// The target slot index.
    pub slot: i16,

    /// The item ID set into the slot.
    pub item_id: String,

    /// The item count set.
    pub item_count: u8,
}

impl InventoryCreativeEvent {
    #[must_use]
    pub const fn new(player: Arc<Player>, slot: i16, item_id: String, item_count: u8) -> Self {
        Self {
            player,
            slot,
            item_id,
            item_count,
            cancelled: false,
        }
    }
}
