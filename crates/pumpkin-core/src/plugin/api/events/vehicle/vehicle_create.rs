use pumpkin_macros::{Event, cancellable};

/// An event that occurs when a vehicle is created.
#[cancellable]
#[derive(Event, Clone)]
pub struct VehicleCreateEvent {
    /// The ID of the vehicle entity.
    pub vehicle_id: i32,
}

impl VehicleCreateEvent {
    #[must_use]
    pub const fn new(vehicle_id: i32) -> Self {
        Self {
            vehicle_id,
            cancelled: false,
        }
    }
}
