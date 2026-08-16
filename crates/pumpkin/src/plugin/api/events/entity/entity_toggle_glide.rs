use pumpkin_macros::{Event, cancellable};

/// An event that occurs when an entity starts or stops gliding with an elytra.
#[cancellable]
#[derive(Event, Clone)]
pub struct EntityToggleGlideEvent {
    /// The ID of the entity.
    pub entity_id: i32,

    /// Whether the entity is now gliding.
    pub is_gliding: bool,
}

impl EntityToggleGlideEvent {
    #[must_use]
    pub const fn new(entity_id: i32, is_gliding: bool) -> Self {
        Self {
            entity_id,
            is_gliding,
            cancelled: false,
        }
    }
}
