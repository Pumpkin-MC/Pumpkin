use pumpkin_macros::{Event, cancellable};

/// An event that occurs when a pig zombie becomes angry.
#[cancellable]
#[derive(Event, Clone)]
pub struct PigZombieAngerEvent {
    /// Pig zombie entity ID.
    pub entity_id: i32,
    /// Target entity ID that caused anger.
    pub target_id: Option<i32>,
    /// New anger level.
    pub new_anger: i32,
}

impl PigZombieAngerEvent {
    #[must_use]
    pub const fn new(entity_id: i32, target_id: Option<i32>, new_anger: i32) -> Self {
        Self {
            entity_id,
            target_id,
            new_anger,
            cancelled: false,
        }
    }
}
