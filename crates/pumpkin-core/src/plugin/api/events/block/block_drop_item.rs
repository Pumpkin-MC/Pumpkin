use pumpkin_data::item_stack::ItemStack;
use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;
use std::sync::Arc;

use crate::{entity::player::Player, world::World};

/// An event that occurs when a block drops items into the world.
#[cancellable]
#[derive(Event, Clone)]
pub struct BlockDropItemEvent {
    pub block_pos: BlockPos,
    pub world: Arc<World>,
    pub player: Option<Arc<Player>>,
    pub items: Vec<ItemStack>,
}

impl BlockDropItemEvent {
    #[must_use]
    pub const fn new(
        block_pos: BlockPos,
        world: Arc<World>,
        player: Option<Arc<Player>>,
        items: Vec<ItemStack>,
    ) -> Self {
        Self {
            block_pos,
            world,
            player,
            items,
            cancelled: false,
        }
    }
}
