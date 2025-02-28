use pumpkin_macros::{Event, cancellable};
use pumpkin_world::block::registry::Block;
use std::sync::Arc;

use crate::entity::player::Player;

use super::BlockEvent;

/// An event that occurs when a player attempts to build on a block.
///
/// This event contains information about the block to build, whether building is allowed,
/// the player attempting to build, and the block being built upon.
#[cancellable]
#[derive(Event, Clone)]
pub struct BlockCanBuildEvent {
    /// The block that the player is attempting to build.
    pub block_to_build: Block,

    /// A boolean indicating whether building is allowed.
    pub buildable: bool,

    /// The player attempting to build.
    pub player: Arc<Player>,

    /// The block being built upon.
    pub block: Block,
}

impl BlockEvent for BlockCanBuildEvent {
    fn get_block(&self) -> &Block {
        &self.block
    }
}
