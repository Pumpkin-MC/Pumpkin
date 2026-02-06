use pumpkin_data::Block;
use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::item::ItemStack;

use super::BlockEvent;

/// An event that occurs when a block dispenses an item.
#[cancellable]
#[derive(Event, Clone)]
pub struct BlockDispenseEvent {
    /// The block that is dispensing.
    pub block: &'static Block,

    /// The position of the block that is dispensing.
    pub block_position: BlockPos,

    /// The world UUID where the dispense occurs.
    pub world_uuid: uuid::Uuid,

    /// The item being dispensed.
    pub item_stack: ItemStack,

    /// The velocity the item will be dispensed with.
    pub velocity: Vector3<f64>,
}

impl BlockDispenseEvent {
    /// Creates a new `BlockDispenseEvent`.
    #[must_use]
    pub const fn new(
        block: &'static Block,
        block_position: BlockPos,
        world_uuid: uuid::Uuid,
        item_stack: ItemStack,
        velocity: Vector3<f64>,
    ) -> Self {
        Self {
            block,
            block_position,
            world_uuid,
            item_stack,
            velocity,
            cancelled: false,
        }
    }
}

impl BlockEvent for BlockDispenseEvent {
    fn get_block(&self) -> &Block {
        self.block
    }
}
