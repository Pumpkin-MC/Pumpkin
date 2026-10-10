use pumpkin_macros::{Event, cancellable};

/// An event that occurs when an entity is set on fire by another entity.
#[cancellable]
#[derive(Event, Clone)]
pub struct EntityCombustByEntityEvent {
    /// Entity ID being combusted.
    pub entity_id: i32,
    /// Combuster entity ID.
    pub combuster_id: i32,
    /// Combust duration in seconds.
    pub duration: f32,
}

impl EntityCombustByEntityEvent {
    #[must_use]
    pub const fn new(entity_id: i32, combuster_id: i32, duration: f32) -> Self {
        Self {
            entity_id,
            combuster_id,
            duration,
            cancelled: false,
        }
    }
}
