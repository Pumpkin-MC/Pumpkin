use pumpkin_macros::{Event, cancellable};

/// An event that occurs when a command is sent to the server console
///
/// This event contains information about the command being executed.
#[cancellable]
#[derive(Event, Clone)]
pub struct ServerCommandEvent {
    /// The command being executed
    pub command: String,
}

impl ServerCommandEvent {
    /// Creates a new instance of `ServerCommandEvent`.
    ///
    /// # Arguments
    /// * `command` - The command being executed.
    ///
    /// # Returns
    /// A new instance of `ServerCommandEvent`.
    #[must_use]
    pub fn new(command: String) -> Self {
        Self {
            command,
            cancelled: false,
        }
    }
}
