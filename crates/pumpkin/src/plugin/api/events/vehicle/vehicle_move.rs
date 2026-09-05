use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::vector3::Vector3;

/// An event that occurs after a vehicle changes position.
///
/// Cancelling the event restores the vehicle's starting position. Changes to
/// [`Self::to`] are applied as the vehicle's final position.
#[cancellable]
#[derive(Event, Clone)]
pub struct VehicleMoveEvent {
    /// The ID of the vehicle entity.
    pub vehicle_id: i32,
    /// Starting position.
    pub from: Vector3<f64>,
    /// Target position.
    pub to: Vector3<f64>,
}

impl VehicleMoveEvent {
    #[must_use]
    pub const fn new(vehicle_id: i32, from: Vector3<f64>, to: Vector3<f64>) -> Self {
        Self {
            vehicle_id,
            from,
            to,
            cancelled: false,
        }
    }
}
