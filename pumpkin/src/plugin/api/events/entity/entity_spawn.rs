use pumpkin_data::entity::EntityType;
use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::vector3::Vector3;
use std::sync::Arc;

use crate::world::World;

use super::EntityEvent;

/// An event that occurs when an entity is spawned in the world.
///
/// If the event is cancelled, the entity will not be spawned.
///
/// This event contains information about the entity being spawned,
/// its type, position, and the world it is being spawned in.
#[cancellable]
#[derive(Event, Clone)]
pub struct EntitySpawnEvent {
    /// The unique ID of the entity being spawned.
    pub entity_id: i32,

    /// The type of entity being spawned.
    pub entity_type: &'static EntityType,

    /// The position where the entity is being spawned.
    pub position: Vector3<f64>,

    /// The world in which the entity is being spawned.
    pub world: Arc<World>,
}

impl EntitySpawnEvent {
    /// Creates a new instance of `EntitySpawnEvent`.
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

impl EntityEvent for EntitySpawnEvent {
    fn get_entity_id(&self) -> i32 {
        self.entity_id
    }

    fn get_entity_type(&self) -> &'static EntityType {
        self.entity_type
    }
}
