use super::PlayerEvent;
use crate::entity::player::Player;
use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

/// An event that occurs when a player changes recipe book settings.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerRecipeBookSettingsChangeEvent {
    /// The player.
    pub player: Arc<Player>,
    /// Recipe book type (crafting, furnace, etc.).
    pub book_type: String,
    /// Whether book is open.
    pub is_open: bool,
    /// Whether filter is active.
    pub is_filtering: bool,
}

impl PlayerRecipeBookSettingsChangeEvent {
    #[must_use]
    pub const fn new(
        player: Arc<Player>,
        book_type: String,
        is_open: bool,
        is_filtering: bool,
    ) -> Self {
        Self {
            player,
            book_type,
            is_open,
            is_filtering,
            cancelled: false,
        }
    }
}

impl PlayerEvent for PlayerRecipeBookSettingsChangeEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
