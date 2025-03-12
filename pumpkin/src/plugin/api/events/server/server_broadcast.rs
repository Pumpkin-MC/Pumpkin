use pumpkin_macros::{Event, cancellable};
use pumpkin_util::text::TextComponent;

/// An event that occurs when something tries to broadcast a message to the server.
///
/// This event contains information about the message being broadcast.
#[cancellable]
#[derive(Event, Clone)]
pub struct ServerBroadcastEvent {
    /// The message being broadcast.
    pub message: TextComponent,
    /// The name of the sender as a TextComponent.
    pub sender: TextComponent,
}

impl ServerBroadcastEvent {
    /// Creates a new instance of `ServerBroadcastEvent`.
    ///
    /// # Arguments
    /// - `message`: The message being broadcast.
    /// - `sender`: The name of the sender as a `TextComponent`.
    ///
    /// # Returns
    /// A new instance of `ServerBroadcastEvent`.
    #[must_use]
    pub fn new(message: TextComponent, sender: TextComponent) -> Self {
        Self {
            message,
            sender,
            cancelled: false,
        }
    }
}
