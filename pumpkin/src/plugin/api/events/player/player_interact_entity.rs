use pumpkin_data::entity::EntityType;
use pumpkin_macros::{Event, cancellable};
use pumpkin_util::Hand;
use std::sync::Arc;

use crate::entity::player::Player;

use super::PlayerEvent;

/// An event that occurs when a player interacts with an entity.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerInteractEntitySimpleEvent {
    /// The player who interacted.
    pub player: Arc<Player>,

    /// The UUID of the entity.
    pub entity_uuid: uuid::Uuid,

    /// The entity type.
    pub entity_type: &'static EntityType,

    /// The hand used.
    pub hand: Hand,
}

impl PlayerInteractEntitySimpleEvent {
    /// Creates a new instance of `PlayerInteractEntitySimpleEvent`.
    pub const fn new(
        player: Arc<Player>,
        entity_uuid: uuid::Uuid,
        entity_type: &'static EntityType,
        hand: Hand,
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

impl PlayerEvent for PlayerInteractEntitySimpleEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
