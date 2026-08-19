use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::vector3::Vector3;

/// An event that occurs when an entity receives knockback.
#[cancellable]
#[derive(Event, Clone)]
pub struct EntityKnockbackEvent {
    /// The ID of the entity receiving knockback.
    pub entity_id: i32,

    /// The ID of the entity that caused the knockback, if any.
    pub hit_by_id: Option<i32>,

    /// The knockback vector.
    pub knockback: Vector3<f64>,
}
