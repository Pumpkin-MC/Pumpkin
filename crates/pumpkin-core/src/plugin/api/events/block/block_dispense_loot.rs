use pumpkin_data::item_stack::ItemStack;
use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;
use std::sync::Arc;

use crate::world::World;

/// An event that occurs when a block dispenses loot (e.g. vault, trial spawner).
#[cancellable]
#[derive(Event, Clone)]
pub struct BlockDispenseLootEvent {
    pub block_pos: BlockPos,
    pub world: Arc<World>,
    pub items: Vec<ItemStack>,
}

impl BlockDispenseLootEvent {
    #[must_use]
    pub const fn new(block_pos: BlockPos, world: Arc<World>, items: Vec<ItemStack>) -> Self {
        Self {
            block_pos,
            world,
            items,
            cancelled: false,
        }
    }
}
