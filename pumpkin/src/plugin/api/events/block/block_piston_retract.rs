use pumpkin_data::Block;
use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;

use super::BlockEvent;

/// An event that occurs when a piston retracts.
///
/// If the event is cancelled, the piston will not retract.
///
/// Matches Bukkit's `BlockPistonRetractEvent`.
#[cancellable]
#[derive(Event, Clone)]
pub struct BlockPistonRetractEvent {
    /// The piston block.
    pub block: &'static Block,

    /// The position of the piston.
    pub block_position: BlockPos,

    /// The direction the piston is retracting from (as a block face index).
    pub direction: u8,

    /// Whether this is a sticky piston.
    pub sticky: bool,
}

impl BlockPistonRetractEvent {
    #[must_use]
    pub const fn new(
        block: &'static Block,
        block_position: BlockPos,
        direction: u8,
        sticky: bool,
    ) -> Self {
        Self {
            block,
            block_position,
            direction,
            sticky,
            cancelled: false,
        }
    }
}

impl BlockEvent for BlockPistonRetractEvent {
    fn get_block(&self) -> &Block {
        self.block
    }
}
