use pumpkin_macros::{Event, cancellable};

/// An event that occurs when a command is executed via remote console (RCON).
#[cancellable]
#[derive(Event, Clone)]
pub struct RemoteServerCommandEvent {
    /// Command line executed.
    pub command: String,
}

impl RemoteServerCommandEvent {
    #[must_use]
    pub const fn new(command: String) -> Self {
        Self {
            command,
            cancelled: false,
        }
    }
}
