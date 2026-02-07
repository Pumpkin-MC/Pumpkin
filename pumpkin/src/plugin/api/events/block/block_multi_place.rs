use pumpkin_data::Block;
use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;
use std::sync::Arc;

use crate::entity::player::Player;

use super::BlockEvent;

/// An event that occurs when multiple blocks are placed at once.
#[cancellable]
#[derive(Event, Clone)]
pub struct BlockMultiPlaceEvent {
    /// The player placing the blocks.
    pub player: Arc<Player>,

    /// The primary block being placed.
    pub block_placed: &'static Block,

    /// The positions where blocks will be placed.
    pub positions: Vec<BlockPos>,
}

impl BlockMultiPlaceEvent {
    /// Creates a new `BlockMultiPlaceEvent`.
    #[must_use]
    pub const fn new(
        player: Arc<Player>,
        block_placed: &'static Block,
        positions: Vec<BlockPos>,
    ) -> Self {
        Self {
            player,
            block_placed,
            positions,
            cancelled: false,
        }
    }
}

impl BlockEvent for BlockMultiPlaceEvent {
    fn get_block(&self) -> &Block {
        self.block_placed
    }
}
