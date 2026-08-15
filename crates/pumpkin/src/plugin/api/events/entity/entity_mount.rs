use pumpkin_macros::{Event, cancellable};

/// An event that occurs when an entity mounts another entity.
#[cancellable]
#[derive(Event, Clone)]
pub struct EntityMountEvent {
    /// The ID of the mounting entity.
    pub entity_id: i32,

    /// The ID of the vehicle entity being mounted.
    pub mount_id: i32,
}

impl EntityMountEvent {
    #[must_use]
    pub const fn new(entity_id: i32, mount_id: i32) -> Self {
        Self {
            entity_id,
            mount_id,
            cancelled: false,
        }
    }
}
