use crate::entity::player::Player;
use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

/// An event that occurs when an item is crafted/upgraded in a smithing table.
#[cancellable]
#[derive(Event, Clone)]
pub struct SmithItemEvent {
    /// The player taking the item from the smithing output.
    pub player: Arc<Player>,

    /// The recipe ID used for smithing.
    pub recipe_id: String,
}

impl SmithItemEvent {
    #[must_use]
    pub const fn new(player: Arc<Player>, recipe_id: String) -> Self {
        Self {
            player,
            recipe_id,
            cancelled: false,
        }
    }
}
