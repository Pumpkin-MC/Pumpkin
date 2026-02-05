use pumpkin_data::Block;
use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;

use super::BlockEvent;

/// An event that occurs when a block spreads to a new position.
#[cancellable]
#[derive(Event, Clone)]
pub struct BlockSpreadEvent {
    /// The source block.
    pub source_block: &'static Block,

    /// The source position.
    pub source_pos: BlockPos,

    /// The block being placed/spread.
    pub block: &'static Block,

    /// The destination position.
    pub block_pos: BlockPos,

    /// The world UUID where the block spread.
    pub world_uuid: uuid::Uuid,
}

impl BlockEvent for BlockSpreadEvent {
    fn get_block(&self) -> &Block {
        self.block
    }
}
