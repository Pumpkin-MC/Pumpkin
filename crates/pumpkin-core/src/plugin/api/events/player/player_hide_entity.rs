use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

use super::PlayerEvent;
use crate::entity::player::Player;

/// An event that occurs when an entity is hidden from a player.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerHideEntityEvent {
    /// The player from whom the entity is hidden.
    pub player: Arc<Player>,

    /// The ID of the hidden entity.
    pub entity_id: i32,
}

impl PlayerEvent for PlayerHideEntityEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
