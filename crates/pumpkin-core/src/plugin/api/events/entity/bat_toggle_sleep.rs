use pumpkin_macros::{Event, cancellable};

/// An event that occurs when a bat toggles its sleep state.
#[cancellable]
#[derive(Event, Clone)]
pub struct BatToggleSleepEvent {
    /// Bat entity ID.
    pub entity_id: i32,
    /// Whether the bat is now awake.
    pub is_awake: bool,
}

impl BatToggleSleepEvent {
    #[must_use]
    pub const fn new(entity_id: i32, is_awake: bool) -> Self {
        Self {
            entity_id,
            is_awake,
            cancelled: false,
        }
    }
}
