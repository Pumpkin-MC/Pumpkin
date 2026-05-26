use pumpkin_macros::Event;

/// An event that fires at the start of every server tick.
#[derive(Event, Clone)]
pub struct ServerTickStartEvent {
    /// 0-indexed number of the tick about to run.
    pub tick: i32,
}

impl ServerTickStartEvent {
    /// Creates a new `ServerTickStartEvent`.
    #[must_use]
    pub const fn new(tick: i32) -> Self {
        Self { tick }
    }
}
