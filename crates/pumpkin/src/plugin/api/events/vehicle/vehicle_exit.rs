use pumpkin_macros::{Event, cancellable};

/// An event that occurs when an entity exits a vehicle.
#[cancellable]
#[derive(Event, Clone)]
pub struct VehicleExitEvent {
    /// The ID of the vehicle entity.
    pub vehicle_id: i32,
    /// The ID of the exiting entity.
    pub exited_id: i32,
}

impl VehicleExitEvent {
    #[must_use]
    pub const fn new(vehicle_id: i32, exited_id: i32) -> Self {
        Self {
            vehicle_id,
            exited_id,
            cancelled: false,
        }
    }
}
