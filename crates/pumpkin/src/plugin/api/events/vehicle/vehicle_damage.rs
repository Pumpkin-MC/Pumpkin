use pumpkin_macros::{Event, cancellable};

/// An event that occurs when a vehicle takes damage.
#[cancellable]
#[derive(Event, Clone)]
pub struct VehicleDamageEvent {
    /// The ID of the vehicle entity.
    pub vehicle_id: i32,
    /// The damage amount.
    pub damage: f32,
    /// ID of attacker if applicable.
    pub attacker_id: Option<i32>,
}

impl VehicleDamageEvent {
    #[must_use]
    pub const fn new(vehicle_id: i32, damage: f32, attacker_id: Option<i32>) -> Self {
        Self {
            vehicle_id,
            damage,
            attacker_id,
            cancelled: false,
        }
    }
}
