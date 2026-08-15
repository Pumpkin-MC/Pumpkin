use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;

/// An event that occurs when fuel is refilled or consumed in a brewing stand.
#[cancellable]
#[derive(Event, Clone)]
pub struct BrewingStandFuelEvent {
    /// The block position of the brewing stand.
    pub block_pos: BlockPos,

    /// The fuel power amount.
    pub fuel_power: u16,
}

impl BrewingStandFuelEvent {
    #[must_use]
    pub const fn new(block_pos: BlockPos, fuel_power: u16) -> Self {
        Self {
            block_pos,
            fuel_power,
            cancelled: false,
        }
    }
}
