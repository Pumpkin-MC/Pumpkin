use pumpkin_data::{Block, BlockDirection};
use pumpkin_macros::{cancellable, Event};
use pumpkin_util::math::position::BlockPos;

use super::BlockEvent;

/// An event that occurs when a block flows from one position to another.
#[cancellable]
#[derive(Event, Clone)]
pub struct BlockFromToEvent {
    /// The block that is flowing.
    pub block: &'static Block,

    /// The position of the source block.
    pub block_pos: BlockPos,

    /// The block at the destination.
    pub to_block: &'static Block,

    /// The position the block is flowing to.
    pub to_pos: BlockPos,

    /// The flow direction.
    pub face: BlockDirection,

    /// The world UUID where the flow occurs.
    pub world_uuid: uuid::Uuid,
}

impl BlockFromToEvent {
    /// Creates a new `BlockFromToEvent`.
    #[must_use]
    pub const fn new(
        block: &'static Block,
        block_pos: BlockPos,
        to_block: &'static Block,
        to_pos: BlockPos,
        face: BlockDirection,
        world_uuid: uuid::Uuid,
    ) -> Self {
        Self {
            block,
            block_pos,
            to_block,
            to_pos,
            face,
            world_uuid,
            cancelled: false,
        }
    }
}

impl BlockEvent for BlockFromToEvent {
    fn get_block(&self) -> &Block {
        self.block
    }
}
