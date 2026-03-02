use pumpkin_data::BlockState;
use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::item::ItemStack;
use std::sync::Arc;

use crate::entity::player::Player;
use crate::world::World;

use super::PlayerEvent;

/// An event that occurs when a player harvests a block.
#[cancellable]
#[derive(Event, Clone)]
pub struct PlayerHarvestBlockEvent {
    /// The player harvesting the block.
    pub player: Arc<Player>,

    /// The world where the block is harvested.
    pub world: Arc<World>,

    /// The harvested block position.
    pub block_pos: BlockPos,

    /// The harvested block state.
    pub block_state: &'static BlockState,

    /// The tool used.
    pub item_stack: ItemStack,

    /// The harvested item drops. Handlers may modify this list.
    pub item_drops: Vec<ItemStack>,

    /// Experience to drop. Handlers may modify this amount.
    pub exp_to_drop: i32,
}

impl PlayerHarvestBlockEvent {
    /// Creates a new instance of `PlayerHarvestBlockEvent`.
    pub const fn new(
        player: Arc<Player>,
        world: Arc<World>,
        block_pos: BlockPos,
        block_state: &'static BlockState,
        item_stack: ItemStack,
        item_drops: Vec<ItemStack>,
        exp_to_drop: i32,
    ) -> Self {
        Self {
            player,
            world,
            block_pos,
            block_state,
            item_stack,
            item_drops,
            exp_to_drop,
            cancelled: false,
        }
    }
}

impl PlayerEvent for PlayerHarvestBlockEvent {
    fn get_player(&self) -> &Arc<Player> {
        &self.player
    }
}
