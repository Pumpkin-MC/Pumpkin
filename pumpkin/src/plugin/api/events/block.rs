use std::sync::Arc;

use pumpkin_macros::{cancellable, event};
use pumpkin_world::block::block_registry::Block;

use crate::entity::player::Player;

pub trait BlockEvent: Send + Sync {
    fn get_block(&self) -> &Block;
}

#[event]
#[cancellable]
pub struct BlockBreakEvent {
    pub player: Option<Arc<Player>>,
    pub block: Block,
    pub exp: u32,
    pub drop: bool,
}

impl BlockBreakEvent {
    #[must_use]
    pub fn new(player: Option<Arc<Player>>, block: Block, exp: u32, drop: bool) -> Self {
        Self {
            player,
            block,
            exp,
            drop,
            cancelled: false,
        }
    }
}

impl BlockEvent for BlockBreakEvent {
    fn get_block(&self) -> &Block {
        &self.block
    }
}

#[event]
#[cancellable]
pub struct BlockBurnEvent {
    pub igniting_block: Block,
    pub block: Block,
}

impl BlockEvent for BlockBurnEvent {
    fn get_block(&self) -> &Block {
        &self.block
    }
}

#[event]
#[cancellable]
pub struct BlockCanBuildEvent {
    pub block_to_build: Block,
    pub buildable: bool,
    pub player: Arc<Player>,
    pub block: Block,
}

impl BlockEvent for BlockCanBuildEvent {
    fn get_block(&self) -> &Block {
        &self.block
    }
}

#[event]
#[cancellable]
pub struct BlockPlaceEvent {
    pub player: Arc<Player>,
    pub block_placed: Block,
    pub block_placed_against: Block,
    pub can_build: bool,
}

impl BlockEvent for BlockPlaceEvent {
    fn get_block(&self) -> &Block {
        &self.block_placed
    }
}
