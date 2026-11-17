use pumpkin_macros::{Event, cancellable};

/// An event that occurs when an entity is primed to explode.
#[cancellable]
#[derive(Event, Clone)]
pub struct ExplosionPrimeEvent {
    /// The ID of the entity.
    pub entity_id: i32,
    /// The explosion radius.
    pub radius: f32,
    /// Whether it creates fire.
    pub fire: bool,
}

impl ExplosionPrimeEvent {
    #[must_use]
    pub const fn new(entity_id: i32, radius: f32, fire: bool) -> Self {
        Self {
            entity_id,
            radius,
            fire,
            cancelled: false,
        }
    }
}
