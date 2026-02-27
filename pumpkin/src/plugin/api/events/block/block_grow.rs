use std::sync::Arc;

use pumpkin_data::Block;
use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;

use crate::world::World;

use super::BlockEvent;

/// An event that occurs when a block grows.
#[cancellable]
#[derive(Event, Clone)]
pub struct BlockGrowEvent {
    /// The world where growth is happening.
    pub world: Arc<World>,

    /// The original block state id.
    pub old_state_id: BlockStateId,

    /// The new block state id to apply.
    pub new_state_id: BlockStateId,

    /// The position of the growing block.
    pub block_pos: BlockPos,
}

impl BlockGrowEvent {
    /// Creates a new `BlockGrowEvent`.
    #[must_use]
    pub const fn new(
        world: Arc<World>,
        old_state_id: BlockStateId,
        new_state_id: BlockStateId,
        block_pos: BlockPos,
    ) -> Self {
        Self {
            world,
            old_state_id,
            new_state_id,
            block_pos,
            cancelled: false,
        }
    }
}

impl BlockEvent for BlockGrowEvent {
    fn get_block(&self) -> &Block {
        Block::from_state_id(self.old_state_id)
    }
}
