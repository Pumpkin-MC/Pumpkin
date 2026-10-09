use super::PlayerEvent;
use crate::entity::player::Player;
use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

/// An event that occurs when a player clicks a recipe in the recipe book.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerRecipeBookClickEvent {
    /// The player.
    pub player: Arc<Player>,
    /// Recipe identifier.
    pub recipe_id: String,
    /// Whether Shift was held (craft max).
    pub make_all: bool,
}

impl PlayerRecipeBookClickEvent {
    #[must_use]
    pub const fn new(player: Arc<Player>, recipe_id: String, make_all: bool) -> Self {
        Self {
            player,
            recipe_id,
            make_all,
            cancelled: false,
        }
    }
}

impl PlayerEvent for PlayerRecipeBookClickEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
