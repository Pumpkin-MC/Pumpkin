use pumpkin_macros::{Event, cancellable};

/// An event that occurs when an entity experiences hunger exhaustion.
#[cancellable]
#[derive(Event, Clone)]
pub struct EntityExhaustionEvent {
    /// The ID of the entity.
    pub entity_id: i32,
    /// The amount of exhaustion added.
    pub exhaustion: f32,
}

impl EntityExhaustionEvent {
    #[must_use]
    pub const fn new(entity_id: i32, exhaustion: f32) -> Self {
        Self {
            entity_id,
            exhaustion,
            cancelled: false,
        }
    }
}
