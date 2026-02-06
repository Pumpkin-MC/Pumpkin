use pumpkin_macros::Event;

/// Fired when a plugin is enabled (loaded successfully).
#[derive(Event, Clone)]
pub struct PluginEnableEvent {
    /// The plugin name.
    pub plugin_name: String,
}

impl PluginEnableEvent {
    #[must_use]
    pub const fn new(plugin_name: String) -> Self {
        Self { plugin_name }
    }
}
