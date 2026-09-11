use crate::world::World;
use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::vector3::Vector3;
use std::sync::Arc;

/// An event that occurs when a creature spawns.
#[cancellable]
#[derive(Event, Clone)]
pub struct CreatureSpawnEvent {
    /// The ID of the spawned creature.
    pub entity_id: i32,

    /// The registry name of the entity type.
    pub entity_type: String,

    /// The position where the creature spawned.
    pub position: Vector3<f64>,

    /// The world in which the creature spawned.
    pub world: Arc<World>,

    /// The reason why the creature spawned.
    pub spawn_reason: String,
}
