use pumpkin_macros::Event;

/// An event that is fired when the server has finished starting up.
///
/// This event is not cancellable as the server has already started.
/// Plugins can use this event to perform post-startup initialization.
#[derive(Event, Clone)]
pub struct ServerStartedEvent {
    /// The number of worlds loaded at startup.
    pub world_count: usize,

    /// The number of plugins loaded at startup.
    pub plugin_count: usize,
}

impl ServerStartedEvent {
    /// Creates a new instance of `ServerStartedEvent`.
    ///
    /// # Arguments
    /// * `world_count` - The number of worlds loaded.
    /// * `plugin_count` - The number of plugins loaded.
    ///
    /// # Returns
    /// A new instance of `ServerStartedEvent`.
    #[must_use]
    pub const fn new(world_count: usize, plugin_count: usize) -> Self {
        Self {
            world_count,
            plugin_count,
        }
    }
}
