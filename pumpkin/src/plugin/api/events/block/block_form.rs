use std::sync::Arc;

use pumpkin_data::Block;
use pumpkin_macros::{Event, cancellable};
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::BlockStateId;

use crate::world::World;

use super::BlockEvent;

/// An event that occurs when a block forms naturally (for example, fluid interactions).
#[cancellable]
#[derive(Event, Clone)]
pub struct BlockFormEvent {
    /// The world where the block formed.
    pub world: Arc<World>,

    /// The original block state id at this position.
    pub old_state_id: BlockStateId,

    /// The new block state id that should be placed.
    pub new_state_id: BlockStateId,

    /// The position of the change.
    pub block_pos: BlockPos,
}

impl BlockFormEvent {
    /// Creates a new `BlockFormEvent`.
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

impl BlockEvent for BlockFormEvent {
    fn get_block(&self) -> &Block {
        Block::from_state_id(self.new_state_id)
    }
}
