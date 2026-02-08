use pumpkin_macros::Event;

/// An event that occurs when a plugin is enabled.
///
/// This event is not cancellable.
///
/// Matches Bukkit's `PluginEnableEvent`.
#[derive(Event, Clone)]
pub struct PluginEnableEvent {
    /// The name of the plugin being enabled.
    pub plugin_name: String,

    /// The version of the plugin being enabled.
    pub plugin_version: String,
}

impl PluginEnableEvent {
    #[must_use]
    pub const fn new(plugin_name: String, plugin_version: String) -> Self {
        Self {
            plugin_name,
            plugin_version,
        }
    }
}
