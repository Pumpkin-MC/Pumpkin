use pumpkin_macros::{Event, cancellable};

use crate::plugin::PluginMetadata;

/// An event that occurs when a plugin is enabled.
///
/// This event wraps the PluginMetadata for the plugin being enabled.
#[cancellable]
#[derive(Event, Clone)]
pub struct ServerPluginEnableEvent {
    /// The metadata of the plugin being enabled.
    pub metadata: PluginMetadata<'static>,
}

impl ServerPluginEnableEvent {
    /// Creates a new instance of `ServerPluginEnableEvent`.
    ///
    /// # Arguments
    /// * `metadata` - The metadata of the plugin being enabled.
    ///
    /// # Returns
    /// A new instance of `ServerPluginEnableEvent`.
    #[must_use]
    pub fn new(metadata: PluginMetadata<'static>) -> Self {
        Self {
            metadata,
            cancelled: false,
        }
    }
}
