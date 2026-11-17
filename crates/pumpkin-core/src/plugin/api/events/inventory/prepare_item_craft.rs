use crate::entity::player::Player;
use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

/// An event that occurs when a recipe is prepared in a crafting matrix.
#[cancellable]
#[derive(Event, Clone)]
pub struct PrepareItemCraftEvent {
    /// The player crafting the item.
    pub player: Arc<Player>,

    /// The recipe ID prepared.
    pub recipe_id: String,
}

impl PrepareItemCraftEvent {
    #[must_use]
    pub const fn new(player: Arc<Player>, recipe_id: String) -> Self {
        Self {
            player,
            recipe_id,
            cancelled: false,
        }
    }
}
