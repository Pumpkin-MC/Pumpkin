use pumpkin_macros::Event;

use crate::plugin::PluginMetadata;

/// An event that fires after a plugin has been successfully loaded.
///
/// This is a notification-only event: the plugin is already active when
/// this event fires and it cannot be cancelled.
#[derive(Event, Clone)]
pub struct PluginLoadEvent {
    /// The name of the loaded plugin.
    pub name: String,
    /// The version of the loaded plugin.
    pub version: String,
}

impl PluginLoadEvent {
    /// Creates a new `PluginLoadEvent` from plugin metadata.
    #[must_use]
    pub fn new(metadata: &PluginMetadata) -> Self {
        Self {
            name: metadata.name.clone(),
            version: metadata.version.clone(),
        }
    }
}
