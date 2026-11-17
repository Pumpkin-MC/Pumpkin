use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

use super::PlayerEvent;
use crate::entity::player::Player;

/// An event that occurs when a player unleashes an entity.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerUnleashEntityEvent {
    /// The player unleashing the entity.
    pub player: Arc<Player>,

    /// The ID of the unleashed entity.
    pub entity_id: i32,
}

impl PlayerEvent for PlayerUnleashEntityEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
