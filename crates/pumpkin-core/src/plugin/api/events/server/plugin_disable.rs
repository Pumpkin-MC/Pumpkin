use pumpkin_macros::{Event, cancellable};

/// An event that occurs when a plugin is disabled.
#[cancellable]
#[derive(Event, Clone)]
pub struct PluginDisableEvent {
    /// Name of the plugin being disabled.
    pub plugin_name: String,
}

impl PluginDisableEvent {
    #[must_use]
    pub const fn new(plugin_name: String) -> Self {
        Self {
            plugin_name,
            cancelled: false,
        }
    }
}
