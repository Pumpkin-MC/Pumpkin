use pumpkin_data::Block;
use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;

use super::BlockEvent;

/// An event that occurs when a block is updated by physics.
#[cancellable]
#[derive(Event, Clone)]
pub struct BlockPhysicsEvent {
    /// The block being updated.
    pub block: &'static Block,

    /// The block position being updated.
    pub block_pos: BlockPos,

    /// The source block that triggered the update.
    pub source_block: &'static Block,

    /// The source block position.
    pub source_pos: BlockPos,

    /// The world UUID where the event occurred.
    pub world_uuid: uuid::Uuid,
}

impl BlockPhysicsEvent {
    /// Creates a new `BlockPhysicsEvent`.
    #[must_use]
    pub const fn new(
        block: &'static Block,
        block_pos: BlockPos,
        source_block: &'static Block,
        source_pos: BlockPos,
        world_uuid: uuid::Uuid,
    ) -> Self {
        Self {
            block,
            block_pos,
            source_block,
            source_pos,
            world_uuid,
            cancelled: false,
        }
    }
}

impl BlockEvent for BlockPhysicsEvent {
    fn get_block(&self) -> &Block {
        self.block
    }
}
