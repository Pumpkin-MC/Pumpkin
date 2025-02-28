use pumpkin_macros::{Event, cancellable};
use pumpkin_world::block::registry::Block;
use std::sync::Arc;

use crate::entity::player::Player;

use super::BlockEvent;

/// An event that occurs when a block is placed.
///
/// This event contains information about the player placing the block, the block being placed,
/// the block being placed against, and whether the player can build.
#[cancellable]
#[derive(Event, Clone)]
pub struct BlockPlaceEvent {
    /// The player placing the block.
    pub player: Arc<Player>,

    /// The block that is being placed.
    pub block_placed: Block,

    /// The block that the new block is being placed against.
    pub block_placed_against: Block,

    /// A boolean indicating whether the player can build.
    pub can_build: bool,
}

impl BlockEvent for BlockPlaceEvent {
    fn get_block(&self) -> &Block {
        &self.block_placed
    }
}
