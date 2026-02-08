use pumpkin_data::Block;
use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;

use super::BlockEvent;

/// An event that occurs when a block is burned.
///
/// This event contains information about the block that ignited the fire and the block that is burning.
#[cancellable]
#[derive(Event, Clone)]
pub struct BlockBurnEvent {
    /// The block that is igniting the fire.
    pub igniting_block: &'static Block,

    /// The block that is burning.
    pub block: &'static Block,

    /// The block position that is burning.
    pub block_pos: BlockPos,

    /// The world UUID where the block burned.
    pub world_uuid: uuid::Uuid,
}

impl BlockEvent for BlockBurnEvent {
    fn get_block(&self) -> &Block {
        self.block
    }
}
