use pumpkin_macros::{Event, cancellable};

/// An event that occurs when world time is skipped.
#[cancellable]
#[derive(Event, Clone)]
pub struct TimeSkipEvent {
    /// Ticks skipped.
    pub skip_amount: i64,
}

impl TimeSkipEvent {
    #[must_use]
    pub const fn new(skip_amount: i64) -> Self {
        Self {
            skip_amount,
            cancelled: false,
        }
    }
}
