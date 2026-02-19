use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

use crate::entity::player::Player;

use super::PlayerEvent;

/// An event that occurs when a player discovers a recipe.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerRecipeDiscoverEvent {
    /// The player discovering the recipe.
    pub player: Arc<Player>,

    /// The recipe key (namespace:id).
    pub recipe_key: String,
}

impl PlayerRecipeDiscoverEvent {
    /// Creates a new instance of `PlayerRecipeDiscoverEvent`.
    pub const fn new(player: Arc<Player>, recipe_key: String) -> Self {
        Self {
            player,
            recipe_key,
            cancelled: false,
        }
    }
}

impl PlayerEvent for PlayerRecipeDiscoverEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
