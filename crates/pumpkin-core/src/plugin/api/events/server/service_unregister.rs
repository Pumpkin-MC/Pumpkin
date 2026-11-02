use pumpkin_macros::{Event, cancellable};

/// An event that occurs when a service is unregistered.
#[cancellable]
#[derive(Event, Clone)]
pub struct ServiceUnregisterEvent {
    /// Name of the service.
    pub service_name: String,
}

impl ServiceUnregisterEvent {
    #[must_use]
    pub const fn new(service_name: String) -> Self {
        Self {
            service_name,
            cancelled: false,
        }
    }
}
