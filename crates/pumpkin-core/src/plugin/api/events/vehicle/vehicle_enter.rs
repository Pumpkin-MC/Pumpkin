use pumpkin_macros::{Event, cancellable};

/// An event that occurs when an entity enters a vehicle.
#[cancellable]
#[derive(Event, Clone)]
pub struct VehicleEnterEvent {
    /// The ID of the vehicle entity.
    pub vehicle_id: i32,
    /// The ID of the entering entity.
    pub entered_id: i32,
}

impl VehicleEnterEvent {
    #[must_use]
    pub const fn new(vehicle_id: i32, entered_id: i32) -> Self {
        Self {
            vehicle_id,
            entered_id,
            cancelled: false,
        }
    }
}
