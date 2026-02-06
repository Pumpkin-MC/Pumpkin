use pumpkin_macros::Event;

/// Fired when a service is registered by a plugin.
#[derive(Event, Clone)]
pub struct ServiceRegisterEvent {
    /// The plugin providing the service.
    pub plugin_name: String,

    /// The service name.
    pub service_name: String,
}

impl ServiceRegisterEvent {
    #[must_use]
    pub const fn new(plugin_name: String, service_name: String) -> Self {
        Self {
            plugin_name,
            service_name,
        }
    }
}
