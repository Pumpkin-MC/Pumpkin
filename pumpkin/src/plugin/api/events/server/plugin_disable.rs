use pumpkin_macros::Event;

/// Fired when a plugin is disabled (unloaded).
#[derive(Event, Clone)]
pub struct PluginDisableEvent {
    /// The plugin name.
    pub plugin_name: String,
}

impl PluginDisableEvent {
    #[must_use]
    #[expect(clippy::missing_const_for_fn)]
    pub fn new(plugin_name: String) -> Self {
        Self { plugin_name }
    }
}
