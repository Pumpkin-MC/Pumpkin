use pumpkin_macros::{Event, cancellable};

/// An event that occurs when a vehicle collides with another entity.
#[cancellable]
#[derive(Event, Clone)]
pub struct VehicleEntityCollisionEvent {
    /// The ID of the vehicle entity.
    pub vehicle_id: i32,
    /// The ID of the collided entity.
    pub collided_entity_id: i32,
}

impl VehicleEntityCollisionEvent {
    #[must_use]
    pub const fn new(vehicle_id: i32, collided_entity_id: i32) -> Self {
        Self {
            vehicle_id,
            collided_entity_id,
            cancelled: false,
        }
    }
}
