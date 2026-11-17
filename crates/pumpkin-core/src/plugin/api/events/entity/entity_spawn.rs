use crate::world::World;
use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::vector3::Vector3;
use std::sync::Arc;

/// An event that occurs when an entity is spawned into a world.
#[cancellable]
#[derive(Event, Clone)]
pub struct EntitySpawnEvent {
    /// The ID of the spawned entity.
    pub entity_id: i32,

    /// The registry name of the entity type.
    pub entity_type: String,

    /// The position where the entity is spawned.
    pub position: Vector3<f64>,

    /// The world in which the entity is spawned.
    pub world: Arc<World>,
}

impl EntitySpawnEvent {
    #[must_use]
    pub const fn new(
        entity_id: i32,
        entity_type: String,
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
