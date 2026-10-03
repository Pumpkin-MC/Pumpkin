use pumpkin_macros::{Event, cancellable};

/// An event that occurs when an entity regains health.
#[cancellable]
#[derive(Event, Clone)]
pub struct EntityRegainHealthEvent {
    /// The ID of the entity regaining health.
    pub entity_id: i32,

    /// The amount of health regained.
    pub amount: f32,
}

impl EntityRegainHealthEvent {
    #[must_use]
    pub const fn new(entity_id: i32, amount: f32) -> Self {
        Self {
            entity_id,
            amount,
            cancelled: false,
        }
    }
}
