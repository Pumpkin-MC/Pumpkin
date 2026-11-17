use pumpkin_macros::{Event, cancellable};

/// An event that occurs when a vehicle is updated per tick.
#[cancellable]
#[derive(Event, Clone)]
pub struct VehicleUpdateEvent {
    /// The ID of the vehicle entity.
    pub vehicle_id: i32,
}

impl VehicleUpdateEvent {
    #[must_use]
    pub const fn new(vehicle_id: i32) -> Self {
        Self {
            vehicle_id,
            cancelled: false,
        }
    }
}
