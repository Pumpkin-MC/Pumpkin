use pumpkin_macros::{Event, cancellable};

/// An event that occurs when a vehicle is destroyed.
#[cancellable]
#[derive(Event, Clone)]
pub struct VehicleDestroyEvent {
    /// The ID of the vehicle entity.
    pub vehicle_id: i32,
    /// ID of attacker if applicable.
    pub attacker_id: Option<i32>,
}

impl VehicleDestroyEvent {
    #[must_use]
    pub const fn new(vehicle_id: i32, attacker_id: Option<i32>) -> Self {
        Self {
            vehicle_id,
            attacker_id,
            cancelled: false,
        }
    }
}
