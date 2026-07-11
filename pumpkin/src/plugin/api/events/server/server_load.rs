use pumpkin_macros::Event;

/// The reason a [`ServerLoadEvent`] fired.
#[derive(Clone, Copy, Debug)]
pub enum LoadType {
    /// The server finished its normal startup sequence.
    Startup,
    /// The server finished a full reload sequence.
    Reload,
}

/// An event that fires once the server has finished loading.
#[derive(Event, Clone)]
pub struct ServerLoadEvent {
    /// Why the server load event was emitted.
    pub load_type: LoadType,
}

impl ServerLoadEvent {
    /// Creates a new `ServerLoadEvent`.
    #[must_use]
    pub const fn new(load_type: LoadType) -> Self {
        Self { load_type }
    }
}
