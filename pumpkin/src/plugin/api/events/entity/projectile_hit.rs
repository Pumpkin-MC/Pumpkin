use pumpkin_data::entity::EntityType;
use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::vector3::Vector3;

use super::EntityEvent;

/// An event that occurs when a projectile hits something (entity or block).
///
/// If the event is cancelled, the hit will not be processed.
///
/// Matches Bukkit's `ProjectileHitEvent`.
#[cancellable]
#[derive(Event, Clone)]
pub struct ProjectileHitEvent {
    /// The unique ID of the projectile entity.
    pub entity_id: i32,

    /// The type of projectile entity.
    pub entity_type: &'static EntityType,

    /// The unique ID of the entity that was hit, if any.
    pub hit_entity_id: Option<i32>,

    /// The location of the impact.
    pub hit_location: Vector3<f64>,
}

impl ProjectileHitEvent {
    #[must_use]
    pub const fn new(
        entity_id: i32,
        entity_type: &'static EntityType,
        hit_entity_id: Option<i32>,
        hit_location: Vector3<f64>,
    ) -> Self {
        Self {
            entity_id,
            entity_type,
            hit_entity_id,
            hit_location,
            cancelled: false,
        }
    }
}

impl EntityEvent for ProjectileHitEvent {
    fn get_entity_id(&self) -> i32 {
        self.entity_id
    }

    fn get_entity_type(&self) -> &'static EntityType {
        self.entity_type
    }
}
