use crate::entity::player::Player;
use pumpkin_macros::Event;
use std::sync::Arc;

/// An event that occurs when a player breaks an item.
#[derive(Event, Clone)]
pub struct PlayerItemBreakEvent {
    /// The player whose item broke.
    pub player: Arc<Player>,

    /// The registry key of the broken item.
    pub item_name: String,
}

impl PlayerItemBreakEvent {
    #[must_use]
    pub const fn new(player: Arc<Player>, item_name: String) -> Self {
        Self { player, item_name }
    }
}
