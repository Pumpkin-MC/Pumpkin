use crate::entity::player::Player;
use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

/// An event that occurs when a player crafts an item.
#[cancellable]
#[derive(Event, Clone)]
pub struct CraftItemEvent {
    /// The player crafting the item.
    pub player: Arc<Player>,

    /// The recipe identifier.
    pub recipe_id: String,
}

impl CraftItemEvent {
    #[must_use]
    pub const fn new(player: Arc<Player>, recipe_id: String) -> Self {
        Self {
            player,
            recipe_id,
            cancelled: false,
        }
    }
}
