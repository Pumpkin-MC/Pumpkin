use pumpkin_macros::{Event, cancellable};

/// An event that occurs when a plugin is enabled.
#[cancellable]
#[derive(Event, Clone)]
pub struct PluginEnableEvent {
    /// Name of the plugin being enabled.
    pub plugin_name: String,
}

impl PluginEnableEvent {
    #[must_use]
    pub const fn new(plugin_name: String) -> Self {
        Self {
            plugin_name,
            cancelled: false,
        }
    }
}
