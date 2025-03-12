use pumpkin_macros::{Event, cancellable};

use crate::plugin::PluginMetadata;

/// An event that occurs when a plugin is disabled.
///
/// This event wraps the PluginMetadata for the plugin being disabled.
#[cancellable]
#[derive(Event, Clone)]
pub struct ServerPluginDisableEvent {
    /// The metadata of the plugin being disabled.
    pub metadata: PluginMetadata<'static>,
}

impl ServerPluginDisableEvent {
    /// Creates a new instance of `ServerPluginDisableEvent`.
    ///
    /// # Arguments
    /// * `metadata` - The metadata of the plugin being disabled.
    ///
    /// # Returns
    /// A new instance of `ServerPluginDisableEvent`.
    #[must_use]
    pub fn new(metadata: PluginMetadata<'static>) -> Self {
        Self {
            metadata,
            cancelled: false,
        }
    }
}
