use pumpkin_data::Block;
use pumpkin_macros::{cancellable, Event};
use pumpkin_util::math::position::BlockPos;

use super::BlockEvent;

/// An event that occurs when a block explodes.
#[cancellable]
#[derive(Event, Clone)]
pub struct BlockExplodeEvent {
    /// The block that caused the explosion.
    pub block: &'static Block,

    /// The position of the block that caused the explosion.
    pub block_position: BlockPos,

    /// The world UUID where the explosion occurred.
    pub world_uuid: uuid::Uuid,

    /// The blocks affected by the explosion.
    pub blocks: Vec<BlockPos>,

    /// The drop yield (0.0 - 1.0).
    pub yield_rate: f32,
}

impl BlockExplodeEvent {
    /// Creates a new `BlockExplodeEvent`.
    #[must_use]
    pub const fn new(
        block: &'static Block,
        block_position: BlockPos,
        world_uuid: uuid::Uuid,
        blocks: Vec<BlockPos>,
        yield_rate: f32,
    ) -> Self {
        Self {
            block,
            block_position,
            world_uuid,
            blocks,
            yield_rate,
            cancelled: false,
        }
    }
}

impl BlockEvent for BlockExplodeEvent {
    fn get_block(&self) -> &Block {
        self.block
    }
}
