use pumpkin_macros::{Event, cancellable};

/// An event that occurs when a raid stops.
#[cancellable]
#[derive(Event, Clone)]
pub struct RaidStopEvent {
    /// Reason for raid stopping.
    pub reason: String,
}

impl RaidStopEvent {
    #[must_use]
    pub const fn new(reason: String) -> Self {
        Self {
            reason,
            cancelled: false,
        }
    }
}
