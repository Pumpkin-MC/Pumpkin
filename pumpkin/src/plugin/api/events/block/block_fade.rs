use pumpkin_data::Block;
use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;

use super::BlockEvent;

/// An event that occurs when a block fades to another state.
#[cancellable]
#[derive(Event, Clone)]
pub struct BlockFadeEvent {
    /// The block that is fading.
    pub block: &'static Block,

    /// The new block after fading.
    pub new_block: &'static Block,

    /// The position of the block.
    pub block_pos: BlockPos,

    /// The world UUID where the block faded.
    pub world_uuid: uuid::Uuid,
}

impl BlockFadeEvent {
    /// Creates a new `BlockFadeEvent`.
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

impl BlockEvent for BlockFadeEvent {
    fn get_block(&self) -> &Block {
        self.block
    }
}
