use pumpkin_macros::Event;

/// An event that is fired when the server is stopping.
///
/// This event is not cancellable — the server shutdown cannot be prevented.
/// Plugins can use this event to perform cleanup tasks such as saving data
/// or disconnecting from external services.
#[derive(Event, Clone)]
pub struct ServerStopEvent {
    /// The reason for the server stopping, if available.
    pub reason: String,
}

impl ServerStopEvent {
    /// Creates a new instance of `ServerStopEvent`.
    ///
    /// # Arguments
    /// * `reason` - The reason for the server stopping.
    ///
    /// # Returns
    /// A new instance of `ServerStopEvent`.
    #[must_use]
    pub const fn new(reason: String) -> Self {
        Self { reason }
    }
}
