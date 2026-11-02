use pumpkin_macros::{Event, cancellable};

/// An event that occurs when a world is saved.
#[cancellable]
#[derive(Event, Clone)]
pub struct WorldSaveEvent {
    /// World name.
    pub world_name: String,
}

impl WorldSaveEvent {
    #[must_use]
    pub const fn new(world_name: String) -> Self {
        Self {
            world_name,
            cancelled: false,
        }
    }
}
