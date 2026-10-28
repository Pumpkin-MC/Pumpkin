use pumpkin_data::item_stack::ItemStack;
use pumpkin_macros::Event;
use pumpkin_util::math::position::BlockPos;
use std::sync::Arc;

use crate::{entity::player::Player, world::World};

/// An event that occurs when a player stops damaging/mining a block before breaking it.
#[derive(Event, Clone)]
pub struct BlockDamageAbortEvent {
    pub player: Arc<Player>,
    pub block_pos: BlockPos,
    pub world: Arc<World>,
    pub item_in_hand: ItemStack,
}

impl BlockDamageAbortEvent {
    #[must_use]
    pub const fn new(
        player: Arc<Player>,
        block_pos: BlockPos,
        world: Arc<World>,
        item_in_hand: ItemStack,
    ) -> Self {
        Self {
            player,
            block_pos,
            world,
            item_in_hand,
        }
    }
}
