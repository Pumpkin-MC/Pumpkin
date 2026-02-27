use std::sync::Arc;

use pumpkin_data::Block;
use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;

use crate::world::World;

use super::BlockEvent;

/// An event that occurs when a block's redstone level changes.
#[cancellable]
#[derive(Event, Clone)]
pub struct BlockRedstoneEvent {
    /// The world where the redstone level changed.
    pub world: Arc<World>,

    /// The block state id whose redstone power changed.
    pub block_state_id: BlockStateId,

    /// The position of the block.
    pub block_pos: BlockPos,

    /// The old redstone current.
    pub old_current: i32,

    /// The new redstone current.
    pub new_current: i32,
}

impl BlockRedstoneEvent {
    /// Creates a new `BlockRedstoneEvent`.
    #[must_use]
    pub const fn new(
        world: Arc<World>,
        block_state_id: BlockStateId,
        block_pos: BlockPos,
        old_current: i32,
        new_current: i32,
    ) -> Self {
        Self {
            world,
            block_state_id,
            block_pos,
            old_current,
            new_current,
            cancelled: false,
        }
    }
}

impl BlockEvent for BlockRedstoneEvent {
    fn get_block(&self) -> &Block {
        Block::from_state_id(self.block_state_id)
    }
}
