use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

use crate::entity::player::Player;

use super::PlayerEvent;

/// An event that occurs when a player interacts with an entity.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerInteractEntityEvent {
    /// The player who interacted.
    pub player: Arc<Player>,

    /// The UUID of the entity.
    pub entity_uuid: uuid::Uuid,

    /// The entity type (registry key).
    pub entity_type: String,

    /// The hand used.
    pub hand: String,
}

impl PlayerInteractEntityEvent {
    /// Creates a new instance of `PlayerInteractEntityEvent`.
    pub fn new(
        player: Arc<Player>,
        entity_uuid: uuid::Uuid,
        entity_type: String,
        hand: String,
    ) -> Self {
        Self {
            player,
            entity_uuid,
            entity_type,
            hand,
            cancelled: false,
        }
    }
}

impl PlayerEvent for PlayerInteractEntityEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
