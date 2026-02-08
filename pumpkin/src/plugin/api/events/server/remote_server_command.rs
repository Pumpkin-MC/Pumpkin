use pumpkin_macros::{Event, cancellable};

/// An event that occurs when a command is received from RCON.
///
/// If the event is cancelled, the command will not be executed.
///
/// Matches Bukkit's `RemoteServerCommandEvent`.
#[cancellable]
#[derive(Event, Clone)]
pub struct RemoteServerCommandEvent {
    /// The command being executed (without the leading slash).
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
