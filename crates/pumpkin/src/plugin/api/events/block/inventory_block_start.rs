use pumpkin_macros::Event;
use pumpkin_util::math::position::BlockPos;
use std::sync::Arc;

use crate::world::World;

/// An event that occurs when an inventory block starts an operation.
#[derive(Event, Clone)]
pub struct InventoryBlockStartEvent {
    pub block_pos: BlockPos,
    pub world: Arc<World>,
}

impl InventoryBlockStartEvent {
    #[must_use]
    pub const fn new(block_pos: BlockPos, world: Arc<World>) -> Self {
        Self { block_pos, world }
    }
}
