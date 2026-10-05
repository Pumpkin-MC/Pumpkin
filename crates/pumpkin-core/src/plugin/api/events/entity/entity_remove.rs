use pumpkin_macros::{Event, cancellable};

/// An event that occurs when an entity is removed from the world.
#[cancellable]
#[derive(Event, Clone)]
pub struct EntityRemoveEvent {
    /// Entity ID.
    pub entity_id: i32,
    /// Removal cause.
    pub cause: String,
}

impl EntityRemoveEvent {
    #[must_use]
    pub const fn new(entity_id: i32, cause: String) -> Self {
        Self {
            entity_id,
            cause,
            cancelled: false,
        }
    }
}
