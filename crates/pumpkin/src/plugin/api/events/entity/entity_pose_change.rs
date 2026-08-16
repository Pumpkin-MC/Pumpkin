use pumpkin_macros::{Event, cancellable};

/// An event that occurs when an entity's pose changes.
#[cancellable]
#[derive(Event, Clone)]
pub struct EntityPoseChangeEvent {
    /// The ID of the entity.
    pub entity_id: i32,
    /// The new pose name.
    pub pose: String,
}

impl EntityPoseChangeEvent {
    #[must_use]
    pub const fn new(entity_id: i32, pose: String) -> Self {
        Self {
            entity_id,
            pose,
            cancelled: false,
        }
    }
}
