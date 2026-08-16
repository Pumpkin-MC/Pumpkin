use pumpkin_macros::{Event, cancellable};

/// An event that occurs when a pig is struck by lightning and transforms.
#[cancellable]
#[derive(Event, Clone)]
pub struct PigZapEvent {
    /// Pig entity ID.
    pub entity_id: i32,
    /// Lightning bolt entity ID.
    pub lightning_id: i32,
    /// Resulting pig zombie entity ID.
    pub pig_zombie_id: i32,
}

impl PigZapEvent {
    #[must_use]
    pub const fn new(entity_id: i32, lightning_id: i32, pig_zombie_id: i32) -> Self {
        Self {
            entity_id,
            lightning_id,
            pig_zombie_id,
            cancelled: false,
        }
    }
}
