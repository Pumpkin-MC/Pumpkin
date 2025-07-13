use pumpkin_data::Block;
use pumpkin_macros::{Event, cancellable};
use std::sync::Arc;

use super::BlockEvent;
use crate::entity::player::Player;
use crate::plugin::player::PlayerEvent;

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
    pub block_placed: &'static Block,

    /// The block that the new block is being placed against.
    pub block_placed_against: &'static Block,

    /// A boolean indicating whether the player can build.
    pub can_build: bool,
}

impl BlockPlaceEvent {
    pub fn new(
        player: Arc<Player>,
        block_placed: &'static Block,
        block_placed_against: &'static Block,
        can_build: bool,
    ) -> Self {
        Self {
            player,
            block_placed,
            block_placed_against,
            can_build,
            cancelled: false,
        }
    }

    #[must_use]
    pub fn get_block_placed(&self) -> &Block {
        self.block_placed
    }

    #[must_use]
    pub fn get_block_against(&self) -> &Block {
        self.block_placed_against
    }

    #[must_use]
    pub fn get_can_build(&self) -> bool {
        self.can_build
    }

    pub fn set_can_build(&mut self, can_build: bool) {
        self.can_build = can_build;
    }
}

impl PlayerEvent for BlockPlaceEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}

impl BlockEvent for BlockPlaceEvent {
    fn get_block(&self) -> &Block {
        self.block_placed
    }
}
