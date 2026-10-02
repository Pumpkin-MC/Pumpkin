use pumpkin_data::item_stack::ItemStack;
use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;
use std::sync::Arc;

use crate::{entity::player::Player, world::World};

/// An event that occurs when a player brushes a brushable block (e.g. suspicious sand/gravel).
#[cancellable]
#[derive(Event, Clone)]
pub struct BlockBrushEvent {
    pub block_pos: BlockPos,
    pub world: Arc<World>,
    pub player: Arc<Player>,
    pub item: ItemStack,
}

impl BlockBrushEvent {
    #[must_use]
    pub const fn new(
        block_pos: BlockPos,
        world: Arc<World>,
        player: Arc<Player>,
        item: ItemStack,
    ) -> Self {
        Self {
            block_pos,
            world,
            player,
            item,
            cancelled: false,
        }
    }
}
