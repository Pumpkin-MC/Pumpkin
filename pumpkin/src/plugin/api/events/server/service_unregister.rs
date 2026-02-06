use pumpkin_macros::Event;

/// Fired when a service is unregistered.
#[derive(Event, Clone)]
pub struct ServiceUnregisterEvent {
    /// The plugin that previously provided the service.
    pub plugin_name: String,

    /// The service name.
    pub service_name: String,
}

impl ServiceUnregisterEvent {
    #[must_use]
    pub const fn new(plugin_name: String, service_name: String) -> Self {
        Self {
            plugin_name,
            service_name,
        }
    }
}
