use pumpkin_data::entity::EntityType;
use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::vector3::Vector3;
use std::sync::Arc;

use crate::world::World;

use super::EntityEvent;

/// An event that occurs when an entity dies.
///
/// If the event is cancelled, the entity will not die (death processing is skipped).
///
/// This event contains information about the dying entity, its position,
/// and the world it is in.
#[cancellable]
#[derive(Event, Clone)]
pub struct EntityDeathEvent {
    /// The unique ID of the entity that died.
    pub entity_id: i32,

    /// The type of entity that died.
    pub entity_type: &'static EntityType,

    /// The position where the entity died.
    pub position: Vector3<f64>,

    /// The world in which the entity died.
    pub world: Arc<World>,
}

impl EntityDeathEvent {
    /// Creates a new instance of `EntityDeathEvent`.
    #[must_use]
    pub fn new(
        entity_id: i32,
        entity_type: &'static EntityType,
        position: Vector3<f64>,
        world: Arc<World>,
    ) -> Self {
        Self {
            entity_id,
            entity_type,
            position,
            world,
            cancelled: false,
        }
    }
}

impl EntityEvent for EntityDeathEvent {
    fn get_entity_id(&self) -> i32 {
        self.entity_id
    }

    fn get_entity_type(&self) -> &'static EntityType {
        self.entity_type
    }
}
