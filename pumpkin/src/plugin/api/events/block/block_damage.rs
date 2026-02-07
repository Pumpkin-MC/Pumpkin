use pumpkin_data::Block;
use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::item::ItemStack;
use std::sync::Arc;

use crate::entity::player::Player;

use super::BlockEvent;

/// An event that occurs when a player starts damaging a block.
#[cancellable]
#[derive(Event, Clone)]
pub struct BlockDamageEvent {
    /// The player damaging the block.
    pub player: Arc<Player>,

    /// The block being damaged.
    pub block: &'static Block,

    /// The position of the block being damaged.
    pub block_position: BlockPos,

    /// The item the player has in their main hand.
    pub item_stack: ItemStack,

    /// Whether the block should break instantly.
    pub insta_break: bool,
}

impl BlockDamageEvent {
    /// Creates a new `BlockDamageEvent`.
    #[must_use]
    pub const fn new(
        player: Arc<Player>,
        block: &'static Block,
        block_position: BlockPos,
        item_stack: ItemStack,
        insta_break: bool,
    ) -> Self {
        Self {
            player,
            block,
            block_position,
            item_stack,
            insta_break,
            cancelled: false,
        }
    }
}

impl BlockEvent for BlockDamageEvent {
    fn get_block(&self) -> &Block {
        self.block
    }
}
