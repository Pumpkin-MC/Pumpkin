use pumpkin_data::Block;
use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::item::ItemStack;
use std::sync::Arc;

use crate::entity::player::Player;

use super::PlayerEvent;

/// An event that occurs when a player harvests a block.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerHarvestBlockEvent {
    /// The player harvesting the block.
    pub player: Arc<Player>,

    /// The harvested block position.
    pub block_pos: BlockPos,

    /// The harvested block.
    pub block: &'static Block,

    /// The tool used.
    pub item_stack: ItemStack,

    /// The harvested item drops. Handlers may modify this list.
    pub item_drops: Vec<ItemStack>,
}

impl PlayerHarvestBlockEvent {
    /// Creates a new instance of `PlayerHarvestBlockEvent`.
    pub const fn new(
        player: Arc<Player>,
        block_pos: BlockPos,
        block: &'static Block,
        item_stack: ItemStack,
        item_drops: Vec<ItemStack>,
    ) -> Self {
        Self {
            player,
            block_pos,
            block,
            item_stack,
            item_drops,
            cancelled: false,
        }
    }
}

impl PlayerEvent for PlayerHarvestBlockEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
