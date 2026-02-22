use pumpkin_data::Block;
use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::item::ItemStack;
use std::sync::Arc;

use crate::entity::player::Player;

use super::BlockEvent;

/// An event that occurs when a block drops items.
#[cancellable]
#[derive(Event, Clone)]
pub struct BlockDropItemEvent {
    /// The player who broke the block.
    pub player: Arc<Player>,

    /// The block that is dropping items.
    pub block: &'static Block,

    /// The position of the block.
    pub block_position: BlockPos,

    /// The items that will be dropped.
    pub items: Vec<ItemStack>,
}

impl BlockDropItemEvent {
    /// Creates a new `BlockDropItemEvent`.
    #[must_use]
    #[expect(clippy::missing_const_for_fn)]
    pub fn new(
        player: Arc<Player>,
        block: &'static Block,
        block_position: BlockPos,
        items: Vec<ItemStack>,
    ) -> Self {
        Self {
            player,
            block,
            block_position,
            items,
            cancelled: false,
        }
    }
}

impl BlockEvent for BlockDropItemEvent {
    fn get_block(&self) -> &Block {
        self.block
    }
}
