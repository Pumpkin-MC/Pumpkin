use pumpkin_data::Block;
use pumpkin_macros::{cancellable, Event};
use pumpkin_util::math::position::BlockPos;
use std::sync::Arc;

use crate::entity::player::Player;

use super::BlockEvent;

/// An event that occurs when a block is fertilized.
#[cancellable]
#[derive(Event, Clone)]
pub struct BlockFertilizeEvent {
    /// The player who fertilized the block.
    pub player: Arc<Player>,

    /// The block being fertilized.
    pub block: &'static Block,

    /// The position of the block.
    pub block_pos: BlockPos,

    /// The blocks that will be affected.
    pub blocks: Vec<BlockPos>,
}

impl BlockFertilizeEvent {
    /// Creates a new `BlockFertilizeEvent`.
    #[must_use]
    pub fn new(
        player: Arc<Player>,
        block: &'static Block,
        block_pos: BlockPos,
        blocks: Vec<BlockPos>,
    ) -> Self {
        Self {
            player,
            block,
            block_pos,
            blocks,
            cancelled: false,
        }
    }
}

impl BlockEvent for BlockFertilizeEvent {
    fn get_block(&self) -> &Block {
        self.block
    }
}
