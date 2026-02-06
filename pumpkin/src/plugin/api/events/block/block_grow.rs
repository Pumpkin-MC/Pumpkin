use pumpkin_data::Block;
use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;

use super::BlockEvent;

/// An event that occurs when a block grows.
#[cancellable]
#[derive(Event, Clone)]
pub struct BlockGrowEvent {
    /// The block that is growing.
    pub block: &'static Block,

    /// The new block after growth.
    pub new_block: &'static Block,

    /// The position of the block.
    pub block_pos: BlockPos,

    /// The world UUID where the block grew.
    pub world_uuid: uuid::Uuid,
}

impl BlockGrowEvent {
    /// Creates a new `BlockGrowEvent`.
    #[must_use]
    pub const fn new(
        block: &'static Block,
        new_block: &'static Block,
        block_pos: BlockPos,
        world_uuid: uuid::Uuid,
    ) -> Self {
        Self {
            block,
            new_block,
            block_pos,
            world_uuid,
            cancelled: false,
        }
    }
}

impl BlockEvent for BlockGrowEvent {
    fn get_block(&self) -> &Block {
        self.block
    }
}
