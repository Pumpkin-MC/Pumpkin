use pumpkin_macros::Event;

/// An event that is fired each server tick (nominally 50ms / 20 TPS).
///
/// This event is not cancellable — the tick cannot be skipped.
/// Plugins can use this event to run periodic logic synchronized with the game loop.
///
/// Performance note: handlers for this event must be lightweight. Heavy computation
/// in tick handlers will directly impact server performance (TPS).
#[derive(Event, Clone)]
pub struct ServerTickEvent {
    /// The current tick number (monotonically increasing from server start).
    pub tick_count: i64,
}

impl ServerTickEvent {
    /// Creates a new instance of `ServerTickEvent`.
    ///
    /// # Arguments
    /// * `tick_count` - The current tick number.
    ///
    /// # Returns
    /// A new instance of `ServerTickEvent`.
    #[must_use]
    pub const fn new(tick_count: i64) -> Self {
        Self { tick_count }
    }
}
