use pumpkin_macros::{Event, cancellable};

/// An event that occurs when an entity catches fire.
#[cancellable]
#[derive(Event, Clone)]
pub struct EntityCombustEvent {
    /// The ID of the entity that caught fire.
    pub entity_id: i32,

    /// The duration in seconds the entity will burn.
    pub duration_secs: f32,
}

impl EntityCombustEvent {
    #[must_use]
    pub const fn new(entity_id: i32, duration_secs: f32) -> Self {
        Self {
            entity_id,
            duration_secs,
            cancelled: false,
        }
    }
}
