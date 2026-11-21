use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::vector3::Vector3;

/// An event that occurs when a projectile hits an entity or block.
#[cancellable]
#[derive(Event, Clone)]
pub struct ProjectileHitEvent {
    /// The ID of the projectile entity.
    pub entity_id: i32,
    /// Hit position.
    pub hit_position: Vector3<f64>,
    /// ID of hit entity if applicable.
    pub hit_entity_id: Option<i32>,
}

impl ProjectileHitEvent {
    #[must_use]
    pub const fn new(
        entity_id: i32,
        hit_position: Vector3<f64>,
        hit_entity_id: Option<i32>,
    ) -> Self {
        Self {
            entity_id,
            hit_position,
            hit_entity_id,
            cancelled: false,
        }
    }
}
