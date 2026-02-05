use pumpkin_data::Block;
use pumpkin_macros::{cancellable, Event};
use pumpkin_util::math::position::BlockPos;

use super::BlockEvent;

/// An event that occurs when a sponge absorbs water.
#[cancellable]
#[derive(Event, Clone)]
pub struct SpongeAbsorbEvent {
    /// The sponge block.
    pub block: &'static Block,

    /// The sponge position.
    pub block_pos: BlockPos,

    /// The world UUID where the absorb occurs.
    pub world_uuid: uuid::Uuid,

    /// The blocks that will be absorbed.
    pub blocks: Vec<BlockPos>,
}

impl SpongeAbsorbEvent {
    /// Creates a new `SpongeAbsorbEvent`.
    #[must_use]
    pub fn new(
        block: &'static Block,
        block_pos: BlockPos,
        world_uuid: uuid::Uuid,
        blocks: Vec<BlockPos>,
    ) -> Self {
        Self {
            block,
            block_pos,
            world_uuid,
            blocks,
            cancelled: false,
        }
    }
}

impl BlockEvent for SpongeAbsorbEvent {
    fn get_block(&self) -> &Block {
        self.block
    }
}
