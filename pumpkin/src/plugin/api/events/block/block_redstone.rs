use pumpkin_data::Block;
use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;

use super::BlockEvent;

/// An event that occurs when a block's redstone level changes.
#[cancellable]
#[derive(Event, Clone)]
pub struct BlockRedstoneEvent {
    /// The block whose redstone power is changing.
    pub block: &'static Block,

    /// The position of the block.
    pub block_pos: BlockPos,

    /// The world UUID where the redstone change occurs.
    pub world_uuid: uuid::Uuid,

    /// The old redstone current.
    pub old_current: i32,

    /// The new redstone current.
    pub new_current: i32,
}

impl BlockRedstoneEvent {
    /// Creates a new `BlockRedstoneEvent`.
    #[must_use]
    pub const fn new(
        block: &'static Block,
        block_pos: BlockPos,
        world_uuid: uuid::Uuid,
        old_current: i32,
        new_current: i32,
    ) -> Self {
        Self {
            block,
            block_pos,
            world_uuid,
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
