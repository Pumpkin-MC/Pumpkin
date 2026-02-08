use pumpkin_data::entity::EntityType;
use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::vector3::Vector3;

use super::EntityEvent;

/// An event that occurs when an entity explodes (TNT, creeper, etc.).
///
/// If the event is cancelled, the explosion will not occur.
///
/// Matches Bukkit's `EntityExplodeEvent`.
#[cancellable]
#[derive(Event, Clone)]
pub struct EntityExplodeEvent {
    /// The unique ID of the entity that is exploding.
    pub entity_id: i32,

    /// The type of entity exploding.
    pub entity_type: &'static EntityType,

    /// The location of the explosion.
    pub location: Vector3<f64>,

    /// The explosion power (radius).
    pub power: f32,
}

impl EntityExplodeEvent {
    #[must_use]
    pub const fn new(
        entity_id: i32,
        entity_type: &'static EntityType,
        location: Vector3<f64>,
        power: f32,
    ) -> Self {
        Self {
            entity_id,
            entity_type,
            location,
            power,
            cancelled: false,
        }
    }
}

impl EntityEvent for EntityExplodeEvent {
    fn get_entity_id(&self) -> i32 {
        self.entity_id
    }

    fn get_entity_type(&self) -> &'static EntityType {
        self.entity_type
    }
}
