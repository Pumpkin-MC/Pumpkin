use pumpkin_macros::{Event, cancellable};

/// An event that occurs when an item entity despawns after aging.
#[cancellable]
#[derive(Event, Clone)]
pub struct ItemDespawnEvent {
    /// The ID of the item entity.
    pub entity_id: i32,
}

impl ItemDespawnEvent {
    #[must_use]
    pub const fn new(entity_id: i32) -> Self {
        Self {
            entity_id,
            cancelled: false,
        }
    }
}
