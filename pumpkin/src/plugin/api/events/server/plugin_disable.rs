use pumpkin_macros::Event;

/// An event that occurs when a plugin is disabled.
///
/// This event is not cancellable.
///
/// Matches Bukkit's `PluginDisableEvent`.
#[derive(Event, Clone)]
pub struct PluginDisableEvent {
    /// The name of the plugin being disabled.
    pub plugin_name: String,

    /// The version of the plugin being disabled.
    pub plugin_version: String,
}

impl PluginDisableEvent {
    #[must_use]
    pub fn new(plugin_name: String, plugin_version: String) -> Self {
        Self {
            plugin_name,
            plugin_version,
        }
    }
}
