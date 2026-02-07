use pumpkin_data::Block;
use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;

use super::BlockEvent;

/// An event that occurs when a block spreads from one location to another.
///
/// This covers liquid flow (water/lava) and dragon egg teleportation.
///
/// If the event is cancelled, the block will not spread.
///
/// Matches Bukkit's `BlockFromToEvent`.
#[cancellable]
#[derive(Event, Clone)]
pub struct BlockFromToEvent {
    /// The source block (the block spreading).
    pub block: &'static Block,

    /// The position of the source block.
    pub block_position: BlockPos,

    /// The destination block (the block being replaced).
    pub to_block: &'static Block,

    /// The position the block is spreading to.
    pub to_position: BlockPos,
}

impl BlockFromToEvent {
    #[must_use]
    pub const fn new(
        block: &'static Block,
        block_position: BlockPos,
        to_block: &'static Block,
        to_position: BlockPos,
    ) -> Self {
        Self {
            block,
            block_position,
            to_block,
            to_position,
            cancelled: false,
        }
    }
}

impl BlockEvent for BlockFromToEvent {
    fn get_block(&self) -> &Block {
        self.block
    }
}
