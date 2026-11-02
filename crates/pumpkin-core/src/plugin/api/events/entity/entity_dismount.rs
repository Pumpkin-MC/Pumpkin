use pumpkin_macros::{Event, cancellable};

/// An event that occurs when an entity dismounts another entity.
#[cancellable]
#[derive(Event, Clone)]
pub struct EntityDismountEvent {
    /// The ID of the dismounting entity.
    pub entity_id: i32,

    /// The ID of the vehicle entity being dismounted.
    pub dismounted_id: i32,
}

impl EntityDismountEvent {
    #[must_use]
    pub const fn new(entity_id: i32, dismounted_id: i32) -> Self {
        Self {
            entity_id,
            dismounted_id,
            cancelled: false,
        }
    }
}
