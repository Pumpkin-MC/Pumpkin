use pumpkin_data::{Block, BlockDirection};
use pumpkin_macros::{cancellable, Event};
use pumpkin_util::math::position::BlockPos;

use super::BlockEvent;

/// An event that occurs when a piston extends.
#[cancellable]
#[derive(Event, Clone)]
pub struct BlockPistonExtendEvent {
    /// The piston block.
    pub block: &'static Block,

    /// The piston position.
    pub block_pos: BlockPos,

    /// The direction the piston is facing.
    pub direction: BlockDirection,

    /// The number of blocks affected.
    pub length: i32,

    /// The blocks being moved.
    pub blocks: Vec<BlockPos>,

    /// The world UUID where the event occurred.
    pub world_uuid: uuid::Uuid,
}

impl BlockPistonExtendEvent {
    /// Creates a new `BlockPistonExtendEvent`.
    #[must_use]
    pub const fn new(
        block: &'static Block,
        block_pos: BlockPos,
        direction: BlockDirection,
        length: i32,
        blocks: Vec<BlockPos>,
        world_uuid: uuid::Uuid,
    ) -> Self {
        Self {
            block,
            block_pos,
            direction,
            length,
            blocks,
            world_uuid,
            cancelled: false,
        }
    }
}

impl BlockEvent for BlockPistonExtendEvent {
    fn get_block(&self) -> &Block {
        self.block
    }
}
