use pumpkin_data::entity::EntityType;
use pumpkin_macros::{Event, cancellable};
use pumpkin_util::Hand;
use pumpkin_util::math::vector3::Vector3;
use std::sync::Arc;

use crate::entity::player::Player;

use super::PlayerEvent;

/// An event that occurs when a player interacts at a specific point on an entity.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerInteractAtEntityEvent {
    /// The player who interacted.
    pub player: Arc<Player>,

    /// The UUID of the entity.
    pub entity_uuid: uuid::Uuid,

    /// The entity type.
    pub entity_type: &'static EntityType,

    /// The hand used.
    pub hand: Hand,

    /// The clicked position on the entity.
    pub clicked_position: Vector3<f32>,
}

impl PlayerInteractAtEntityEvent {
    /// Creates a new instance of `PlayerInteractAtEntityEvent`.
    pub const fn new(
        player: Arc<Player>,
        entity_uuid: uuid::Uuid,
        entity_type: &'static EntityType,
        hand: Hand,
        clicked_position: Vector3<f32>,
    ) -> Self {
        Self {
            player,
            entity_uuid,
            entity_type,
            hand,
            clicked_position,
            cancelled: false,
        }
    }
}

impl PlayerEvent for PlayerInteractAtEntityEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
