use pumpkin_macros::{Event, cancellable};

/// An event that occurs when an entity transforms into another entity.
#[cancellable]
#[derive(Event, Clone)]
pub struct EntityTransformEvent {
    /// The ID of the original entity.
    pub entity_id: i32,

    /// The ID of the new transformed entity.
    pub new_entity_id: i32,

    /// The reason for transformation.
    pub transform_reason: String,
}

impl EntityTransformEvent {
    #[must_use]
    pub const fn new(entity_id: i32, new_entity_id: i32, transform_reason: String) -> Self {
        Self {
            entity_id,
            new_entity_id,
            transform_reason,
            cancelled: false,
        }
    }
}
