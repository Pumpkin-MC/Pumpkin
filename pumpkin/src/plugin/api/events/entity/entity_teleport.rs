use pumpkin_data::entity::EntityType;
use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::vector3::Vector3;
use std::sync::Arc;

use crate::world::World;

use super::EntityEvent;

/// An event that occurs when an entity teleports.
///
/// If the event is cancelled, the teleport will not occur.
///
/// Matches Bukkit's `EntityTeleportEvent`.
#[cancellable]
#[derive(Event, Clone)]
pub struct EntityTeleportEvent {
    /// The unique ID of the entity teleporting.
    pub entity_id: i32,

    /// The type of entity teleporting.
    pub entity_type: &'static EntityType,

    /// The position the entity is teleporting from.
    pub from: Vector3<f64>,

    /// The position the entity is teleporting to.
    pub to: Vector3<f64>,

    /// The world the entity is in.
    pub world: Arc<World>,
}

impl EntityTeleportEvent {
    #[must_use]
    pub const fn new(
        entity_id: i32,
        entity_type: &'static EntityType,
        from: Vector3<f64>,
        to: Vector3<f64>,
        world: Arc<World>,
    ) -> Self {
        Self {
            entity_id,
            entity_type,
            from,
            to,
            world,
            cancelled: false,
        }
    }
}

impl EntityEvent for EntityTeleportEvent {
    fn get_entity_id(&self) -> i32 {
        self.entity_id
    }

    fn get_entity_type(&self) -> &'static EntityType {
        self.entity_type
    }
}
