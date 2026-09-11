use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;

/// An event that occurs when a vehicle collides with a block.
#[cancellable]
#[derive(Event, Clone)]
pub struct VehicleBlockCollisionEvent {
    /// The ID of the vehicle entity.
    pub vehicle_id: i32,
    /// Position of the collided block.
    pub block_pos: BlockPos,
}

impl VehicleBlockCollisionEvent {
    #[must_use]
    pub const fn new(vehicle_id: i32, block_pos: BlockPos) -> Self {
        Self {
            vehicle_id,
            block_pos,
            cancelled: false,
        }
    }
}
