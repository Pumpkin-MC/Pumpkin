use pumpkin_macros::{Event, cancellable};

/// An event that occurs when an entity is knocked back by another entity.
#[cancellable]
#[derive(Event, Clone)]
pub struct EntityKnockbackByEntityEvent {
    /// Entity ID receiving knockback.
    pub entity_id: i32,
    /// Entity ID causing knockback.
    pub hit_by_id: i32,
    /// Knockback force.
    pub force: f64,
    /// X velocity change.
    pub x: f64,
    /// Z velocity change.
    pub z: f64,
}

impl EntityKnockbackByEntityEvent {
    #[must_use]
    pub const fn new(entity_id: i32, hit_by_id: i32, force: f64, x: f64, z: f64) -> Self {
        Self {
            entity_id,
            hit_by_id,
            force,
            x,
            z,
            cancelled: false,
        }
    }
}
