use pumpkin_data::item_stack::ItemStack;
use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;
use std::sync::Arc;

use crate::world::World;

/// An event that occurs when a block cooks an item (e.g. campfire).
#[cancellable]
#[derive(Event, Clone)]
pub struct BlockCookEvent {
    pub block_pos: BlockPos,
    pub world: Arc<World>,
    pub source: ItemStack,
    pub result: ItemStack,
}

impl BlockCookEvent {
    #[must_use]
    pub const fn new(
        block_pos: BlockPos,
        world: Arc<World>,
        source: ItemStack,
        result: ItemStack,
    ) -> Self {
        Self {
            block_pos,
            world,
            source,
            result,
            cancelled: false,
        }
    }
}
