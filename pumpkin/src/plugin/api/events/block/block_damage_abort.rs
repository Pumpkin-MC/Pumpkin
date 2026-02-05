use pumpkin_data::Block;
use pumpkin_macros::{cancellable, Event};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::item::ItemStack;
use std::sync::Arc;

use crate::entity::player::Player;

use super::BlockEvent;

/// An event that occurs when a player stops damaging a block.
#[cancellable]
#[derive(Event, Clone)]
pub struct BlockDamageAbortEvent {
    /// The player who stopped damaging the block.
    pub player: Arc<Player>,

    /// The block that was being damaged.
    pub block: &'static Block,

    /// The position of the block that was being damaged.
    pub block_position: BlockPos,

    /// The item the player has in their main hand.
    pub item_stack: ItemStack,
}

impl BlockDamageAbortEvent {
    /// Creates a new `BlockDamageAbortEvent`.
    #[must_use]
    pub const fn new(
        player: Arc<Player>,
        block: &'static Block,
        block_position: BlockPos,
        item_stack: ItemStack,
    ) -> Self {
        Self {
            player,
            block,
            block_position,
            item_stack,
            cancelled: false,
        }
    }
}

impl BlockEvent for BlockDamageAbortEvent {
    fn get_block(&self) -> &Block {
        self.block
    }
}
