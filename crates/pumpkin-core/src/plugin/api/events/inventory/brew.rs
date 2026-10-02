use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;

/// An event that occurs when a brewing stand finishes brewing potion(s).
#[cancellable]
#[derive(Event, Clone)]
pub struct BrewEvent {
    /// The block position of the brewing stand.
    pub block_pos: BlockPos,

    /// The remaining fuel power.
    pub fuel_level: u8,
}

impl BrewEvent {
    #[must_use]
    pub const fn new(block_pos: BlockPos, fuel_level: u8) -> Self {
        Self {
            block_pos,
            fuel_level,
            cancelled: false,
        }
    }
}
