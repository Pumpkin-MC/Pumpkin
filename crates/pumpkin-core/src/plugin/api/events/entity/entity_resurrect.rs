use pumpkin_macros::{Event, cancellable};

/// An event that occurs when an entity is saved from death by Totem of Undying.
#[cancellable]
#[derive(Event, Clone)]
pub struct EntityResurrectEvent {
    /// The ID of the resurrected entity.
    pub entity_id: i32,
}

impl EntityResurrectEvent {
    #[must_use]
    pub const fn new(entity_id: i32) -> Self {
        Self {
            entity_id,
            cancelled: false,
        }
    }
}
