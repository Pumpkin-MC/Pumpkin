use pumpkin_macros::{Event, cancellable};

/// An event that occurs when an entity is unleashed.
#[cancellable]
#[derive(Event, Clone)]
pub struct EntityUnleashEvent {
    /// Entity ID being unleashed.
    pub entity_id: i32,
    /// Unleash reason.
    pub reason: String,
}

impl EntityUnleashEvent {
    #[must_use]
    pub const fn new(entity_id: i32, reason: String) -> Self {
        Self {
            entity_id,
            reason,
            cancelled: false,
        }
    }
}
