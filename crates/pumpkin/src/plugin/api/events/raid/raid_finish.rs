use pumpkin_macros::{Event, cancellable};

/// An event that occurs when a raid finishes.
#[cancellable]
#[derive(Event, Clone)]
pub struct RaidFinishEvent {
    /// Whether victory was achieved by players.
    pub victory: bool,
}

impl RaidFinishEvent {
    #[must_use]
    pub const fn new(victory: bool) -> Self {
        Self {
            victory,
            cancelled: false,
        }
    }
}
