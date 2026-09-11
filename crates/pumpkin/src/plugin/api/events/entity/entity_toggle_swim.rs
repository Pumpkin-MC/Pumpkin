use pumpkin_macros::{Event, cancellable};

/// An event that occurs when an entity starts or stops swimming.
#[cancellable]
#[derive(Event, Clone)]
pub struct EntityToggleSwimEvent {
    /// The ID of the entity.
    pub entity_id: i32,
    /// Whether the entity is swimming.
    pub is_swimming: bool,
}

impl EntityToggleSwimEvent {
    #[must_use]
    pub const fn new(entity_id: i32, is_swimming: bool) -> Self {
        Self {
            entity_id,
            is_swimming,
            cancelled: false,
        }
    }
}
