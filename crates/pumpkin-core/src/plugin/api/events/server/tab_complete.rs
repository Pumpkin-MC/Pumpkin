use pumpkin_macros::{Event, cancellable};

/// An event that occurs when tab completions are requested.
#[cancellable]
#[derive(Event, Clone)]
pub struct TabCompleteEvent {
    /// Buffer / text being completed.
    pub buffer: String,
    /// Completion suggestions.
    pub completions: Vec<String>,
}

impl TabCompleteEvent {
    #[must_use]
    pub const fn new(buffer: String, completions: Vec<String>) -> Self {
        Self {
            buffer,
            completions,
            cancelled: false,
        }
    }
}
