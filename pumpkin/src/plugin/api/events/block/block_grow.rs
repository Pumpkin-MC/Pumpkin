use pumpkin_data::Block;
use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;

use super::BlockEvent;

/// An event that occurs when a block grows naturally.
///
/// This covers crops growing, saplings growing into trees,
/// cactus/sugar cane/bamboo extending, etc.
///
/// If the event is cancelled, the growth will not occur.
///
/// Matches Bukkit's `BlockGrowEvent`.
#[cancellable]
#[derive(Event, Clone)]
pub struct BlockGrowEvent {
    /// The block that is growing.
    pub block: &'static Block,

    /// The position of the block.
    pub block_position: BlockPos,

    /// The new block state after growth.
    pub new_block: &'static Block,
}

impl BlockGrowEvent {
    #[must_use]
    pub const fn new(
        block: &'static Block,
        block_position: BlockPos,
        new_block: &'static Block,
    ) -> Self {
        Self {
            block,
            block_position,
            new_block,
            cancelled: false,
        }
    }
}

impl BlockEvent for BlockGrowEvent {
    fn get_block(&self) -> &Block {
        self.block
    }
}
