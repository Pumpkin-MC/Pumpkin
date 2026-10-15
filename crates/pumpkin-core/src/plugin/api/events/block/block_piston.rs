use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;

/// An event that occurs when a piston extends.
#[cancellable]
#[derive(Event, Clone)]
pub struct BlockPistonExtendEvent {
    pub block_pos: BlockPos,
    pub direction: String,
}

impl BlockPistonExtendEvent {
    #[must_use]
    pub const fn new(block_pos: BlockPos, direction: String) -> Self {
        Self {
            block_pos,
            direction,
            cancelled: false,
        }
    }
}

/// An event that occurs when a piston retracts.
#[cancellable]
#[derive(Event, Clone)]
pub struct BlockPistonRetractEvent {
    pub block_pos: BlockPos,
    pub direction: String,
}

impl BlockPistonRetractEvent {
    #[must_use]
    pub const fn new(block_pos: BlockPos, direction: String) -> Self {
        Self {
            block_pos,
            direction,
            cancelled: false,
        }
    }
}
