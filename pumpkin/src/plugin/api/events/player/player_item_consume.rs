use crate::entity::player::Player;
use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

/// An event that occurs when a player consumes an item (food, potion, etc.).
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerItemConsumeEvent {
    /// The player consuming the item.
    pub player: Arc<Player>,

    /// The registry name of the item being consumed.
    pub item_name: String,
}

impl PlayerItemConsumeEvent {
    #[must_use]
    pub const fn new(player: Arc<Player>, item_name: String) -> Self {
        Self {
            player,
            item_name,
            cancelled: false,
        }
    }
}
