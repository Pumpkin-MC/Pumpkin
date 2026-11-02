use pumpkin_macros::{Event, cancellable};

/// Base event triggered when a vehicle collides.
#[cancellable]
#[derive(Event, Clone)]
pub struct VehicleCollisionEvent {
    /// The ID of the vehicle entity.
    pub vehicle_id: i32,
}

impl VehicleCollisionEvent {
    #[must_use]
    pub const fn new(vehicle_id: i32) -> Self {
        Self {
            vehicle_id,
            cancelled: false,
        }
    }
}
