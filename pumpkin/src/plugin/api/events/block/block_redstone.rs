use pumpkin_data::Block;
use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;

use super::BlockEvent;

/// An event that occurs when a block's redstone current changes.
///
/// If the event is cancelled, the redstone signal change will not propagate.
///
/// Matches Bukkit's `BlockRedstoneEvent`.
#[cancellable]
#[derive(Event, Clone)]
pub struct BlockRedstoneEvent {
    /// The block whose redstone state is changing.
    pub block: &'static Block,

    /// The position of the block.
    pub block_position: BlockPos,

    /// The old redstone current level (0-15).
    pub old_current: u8,

    /// The new redstone current level (0-15).
    pub new_current: u8,
}

impl BlockRedstoneEvent {
    #[must_use]
    pub const fn new(
        block: &'static Block,
        block_position: BlockPos,
        old_current: u8,
        new_current: u8,
    ) -> Self {
        Self {
            block,
            block_position,
            old_current,
            new_current,
            cancelled: false,
        }
    }
}

impl BlockEvent for BlockRedstoneEvent {
    fn get_block(&self) -> &Block {
        self.block
    }
}
