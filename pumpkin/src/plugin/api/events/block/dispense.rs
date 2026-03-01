use pumpkin_data::{Block, BlockState};
use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::item::ItemStack;
use std::sync::Arc;

use crate::world::World;

use super::BlockEvent;

/// An event that occurs when a block dispenses an item.
#[cancellable]
#[derive(Event, Clone)]
pub struct BlockDispenseEvent {
    /// The world where the block is dispensing.
    pub world: Arc<World>,

    /// The block state that is dispensing.
    pub block_state: &'static BlockState,

    /// The position of the block that is dispensing.
    pub block_position: BlockPos,

    /// The item being dispensed.
    pub item_stack: ItemStack,

    /// The velocity the item will be dispensed with.
    pub velocity: Vector3<f64>,
}

impl BlockDispenseEvent {
    /// Creates a new `BlockDispenseEvent`.
    #[must_use]
    pub const fn new(
        world: Arc<World>,
        block_state: &'static BlockState,
        block_position: BlockPos,
        item_stack: ItemStack,
        velocity: Vector3<f64>,
    ) -> Self {
        Self {
            world,
            block_state,
            block_position,
            item_stack,
            velocity,
            cancelled: false,
        }
    }
}

impl BlockEvent for BlockDispenseEvent {
    fn get_block(&self) -> &Block {
        Block::from_state_id(self.block_state.id)
    }
}
