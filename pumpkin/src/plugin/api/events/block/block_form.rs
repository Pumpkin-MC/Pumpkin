use pumpkin_data::Block;
use pumpkin_macros::{cancellable, Event};
use pumpkin_util::math::position::BlockPos;

use super::BlockEvent;

/// An event that occurs when a block forms naturally (e.g., fluid or other transformations).
#[cancellable]
#[derive(Event, Clone)]
pub struct BlockFormEvent {
    /// The block that is forming.
    pub new_block: &'static Block,

    /// The old block being replaced.
    pub old_block: &'static Block,

    /// The position of the block.
    pub block_pos: BlockPos,

    /// The world UUID where the block formed.
    pub world_uuid: uuid::Uuid,
}

impl BlockFormEvent {
    /// Creates a new `BlockFormEvent`.
    #[must_use]
    pub const fn new(
        new_block: &'static Block,
        old_block: &'static Block,
        block_pos: BlockPos,
        world_uuid: uuid::Uuid,
    ) -> Self {
        Self {
            new_block,
            old_block,
            block_pos,
            world_uuid,
            cancelled: false,
        }
    }
}

impl BlockEvent for BlockFormEvent {
    fn get_block(&self) -> &Block {
        self.new_block
    }
}
