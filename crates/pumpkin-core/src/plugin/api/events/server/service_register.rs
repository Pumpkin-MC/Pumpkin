use pumpkin_macros::{Event, cancellable};

/// An event that occurs when a service is registered.
#[cancellable]
#[derive(Event, Clone)]
pub struct ServiceRegisterEvent {
    /// Name of the service.
    pub service_name: String,
}

impl ServiceRegisterEvent {
    #[must_use]
    pub const fn new(service_name: String) -> Self {
        Self {
            service_name,
            cancelled: false,
        }
    }
}
