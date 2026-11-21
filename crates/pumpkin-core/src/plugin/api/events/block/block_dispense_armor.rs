use pumpkin_data::item_stack::ItemStack;
use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;
use std::sync::Arc;

use crate::{entity::EntityBase, world::World};

/// An event that occurs when a dispenser equips armor on an entity.
#[cancellable]
#[derive(Event, Clone)]
pub struct BlockDispenseArmorEvent {
    pub block_pos: BlockPos,
    pub world: Arc<World>,
    pub target: Arc<dyn EntityBase>,
    pub item: ItemStack,
}

impl BlockDispenseArmorEvent {
    #[must_use]
    pub const fn new(
        block_pos: BlockPos,
        world: Arc<World>,
        target: Arc<dyn EntityBase>,
        item: ItemStack,
    ) -> Self {
        Self {
            block_pos,
            world,
            target,
            item,
            cancelled: false,
        }
    }
}
