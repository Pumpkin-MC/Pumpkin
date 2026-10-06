use std::sync::Arc;

use pumpkin_data::Block;
use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;

use crate::world::World;

use super::BlockEvent;

/// An event that occurs when a block is burned.
///
/// This event contains information about the world the block is in, its position, the block that
/// ignited the fire and the block that is burning.
#[cancellable]
#[derive(Event, Clone)]
pub struct BlockBurnEvent {
    /// The world the block is burning in.
    pub world: Arc<World>,

    /// The block that is igniting the fire.
    pub igniting_block: &'static Block,

    /// The block that is burning.
    pub block: &'static Block,

    /// The position of the burning block.
    pub block_pos: BlockPos,
}

impl BlockEvent for BlockBurnEvent {
    fn get_block(&self) -> &Block {
        self.block
    }
}
