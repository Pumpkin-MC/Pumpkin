use pumpkin_data::item_stack::ItemStack;
use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;
use std::sync::Arc;

use crate::world::World;

/// An event that occurs when a campfire starts cooking an item.
#[cancellable]
#[derive(Event, Clone)]
pub struct CampfireStartEvent {
    pub block_pos: BlockPos,
    pub world: Arc<World>,
    pub item: ItemStack,
    pub slot: u8,
    pub cooking_time: i32,
}

impl CampfireStartEvent {
    #[must_use]
    pub const fn new(
        block_pos: BlockPos,
        world: Arc<World>,
        item: ItemStack,
        slot: u8,
        cooking_time: i32,
    ) -> Self {
        Self {
            block_pos,
            world,
            item,
            slot,
            cooking_time,
            cancelled: false,
        }
    }
}
