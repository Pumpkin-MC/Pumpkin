use crate::entity::player::Player;
use pumpkin_macros::Event;
use std::sync::Arc;

/// An event that occurs when items are placed into a smithing table.
#[derive(Event, Clone)]
pub struct PrepareSmithingEvent {
    /// The player using the smithing table.
    pub player: Arc<Player>,

    /// The resulting item ID prepared in the output slot.
    pub result_item: Option<String>,
}

impl PrepareSmithingEvent {
    #[must_use]
    pub const fn new(player: Arc<Player>, result_item: Option<String>) -> Self {
        Self {
            player,
            result_item,
        }
    }
}
